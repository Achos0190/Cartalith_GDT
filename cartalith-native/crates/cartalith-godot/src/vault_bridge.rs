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

/// [`err`] plus VE-4's `kind` (`cartalith_vault::Error::write_kind`): the refusal's class as a
/// stable word, so the editor offers Keep mine / Take theirs only for `"conflict"` and not for a
/// read-only file, and never has to match on the English message.
fn write_err(e: &cartalith_vault::Error) -> VarDictionary {
    let mut d = err(e);
    d.set("kind", e.write_kind());
    // A conflict also names the hash the file has NOW, so "Keep mine" can write against exactly the
    // version the person was told about: a second outside change after the banner refuses again.
    if let cartalith_vault::Error::SourceChanged { actual, .. } = e {
        d.set("actual", actual.as_str());
    }
    d
}

/// The clock seam's live side (Ruling BF): the system's **local** date and
/// time, as Obsidian's `{{date}}`/`{{time}}` use it. `None` only if Godot
/// hands back something that is not a real date.
fn local_now() -> Option<cartalith_vault::template::DateTime> {
    let d = godot::classes::Time::singleton().get_datetime_dict_from_system();
    let g = |k: &str| d.get(k).and_then(|v| v.try_to::<i64>().ok());
    cartalith_vault::template::DateTime::new(
        g("year")? as i32,
        g("month")? as u32,
        g("day")? as u32,
        g("hour")? as u32,
        g("minute")? as u32,
        g("second")? as u32,
    )
}

/// The success shape every method here returns, `{"ok": true, "error": ""}`,
/// the counterpart of [`err`]; a method adds its own keys to it. `pub(crate)`
/// for the same reason as [`err`]: `vault_saf.rs` returns the identical shape.
pub(crate) fn ok() -> VarDictionary {
    vdict! { "ok" => true, "error" => "" }
}

/// `SAVEFILE_COMPAT.md` §13.3.5's *"report it"*, in one sentence, or `None`
/// when there is nothing to report — which is the normal case and the common
/// one.
///
/// `pub(crate)` because the archive path (`project_bridge.rs`'s `project_open`)
/// wants the identical sentence and must not grow a second phrasing of it.
/// **It does not call it yet.** Measured 2026-09-06:
/// `grep -rn "unresolvable_kind_report" crates/cartalith-godot/src/ -l` names
/// **this file only**, and `grep -c` in it gives **8** — this comment, the
/// definition, the one call in [`WorldGen::vault_restore_state`], and five in
/// the test. `project_bridge.rs` has none.
///
/// That is a gap in the *reporting* on the archive path, not in the keeping:
/// the links themselves survive `project_open` either way now, because the
/// keeping is in `cartalith_vault::LinkKind` and not here. Wiring it is one
/// call beside that function's existing `Err(e) => godot_warn!` arm, in a file
/// this change did not own.
pub(crate) fn unresolvable_kind_report(store: &cartalith_vault::LinkStore) -> Option<String> {
    let odd = store.unresolvable_links();
    if odd.is_empty() {
        return None;
    }
    let mut kinds: Vec<&str> = odd.iter().map(|l| l.entity_kind.as_str()).collect();
    kinds.sort_unstable();
    kinds.dedup();
    Some(format!(
        "{} of this project's {} knowledge links name an entity kind this build does not know ({}); \
         they are kept, listed in the vault window's overview under the name they were saved with, \
         and written back unchanged. Nothing was discarded.",
        odd.len(),
        store.links.len(),
        kinds.join(", ")
    ))
}

/// `"settlement"`/`"province"`/`"continent"` from GDScript into the enum.
/// An unknown string is `None` and every caller turns that into an `error`,
/// never a default — silently treating a typo as "settlement" would attach a
/// note to the wrong thing.
///
/// **This is the *input* path, and it is the one place refusing is right.**
/// A kind arriving here is a live request to attach or to list, so there is a
/// user in front of it and an `error` reaches them. A kind arriving from a
/// *file* is the opposite case: nobody is there, the writer is gone, and
/// refusing costs somebody's filing — so `SAVEFILE_COMPAT.md` §13.3.5 keeps it
/// instead, and `cartalith_vault::LinkKind` is where that happens. The two
/// answers differ on purpose; see
/// [`WorldGen::vault_restore_state`] for the other half.
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
    /// opened by this call). A non-positive `limit` uses `cartalith_vault::LIST_CAP`
    /// (12 000 since owner answer K9, 2026-10-26; it was 2 000).
    #[func]
    fn vault_list_files(&self, limit: i64) -> PackedStringArray {
        let limit = if limit > 0 { limit as usize } else { cartalith_vault::LIST_CAP };
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

    /// The browse panel's edit path: a whole file's raw text plus the hash it
    /// was read at, for a note picked but not attached to any entity.
    /// `{ok, error, text, hash}`. `hash` must be handed back to
    /// `vault_write_file` unchanged — same pairing as
    /// `vault_preview_section_write`/`vault_write_section`, just with no
    /// link and no section splice in between.
    #[func]
    fn vault_read_file_for_edit(&self, rel: GString) -> VarDictionary {
        match self.vault.read_for_edit(&rel.to_string()) {
            Ok((text, hash)) => {
                let mut d = ok();
                d.set("text", text);
                d.set("hash", hash);
                d
            }
            Err(e) => err(e),
        }
    }

    /// Writes a whole file with no attached link — the browse panel's
    /// simpler cousin of `vault_write_section`. `expect_hash` must be the
    /// hash `vault_read_file_for_edit` returned; refuses with the file
    /// unchanged if the file was edited since. `{ok, error, hash}` — the
    /// returned `hash` is what the next write of the same session should
    /// guard against.
    #[func]
    fn vault_write_file(&mut self, rel: GString, text: GString, expect_hash: GString) -> VarDictionary {
        match self.vault.write_file(&rel.to_string(), &text.to_string(), &expect_hash.to_string()) {
            Ok(hash) => {
                let mut d = ok();
                d.set("hash", hash);
                d
            }
            Err(e) => write_err(&e),
        }
    }

    /// Creates a note that does not exist (VE-4: **Keep both**, and **Recreate** after a delete).
    /// Never overwrites: an existing `rel` answers `kind: "exists"`. `{ok, error, kind, hash}`.
    #[func]
    fn vault_create_file(&mut self, rel: GString, text: GString) -> VarDictionary {
        match self.vault.create_file(&rel.to_string(), &text.to_string()) {
            Ok(hash) => {
                let mut d = ok();
                d.set("hash", hash);
                d
            }
            Err(e) => write_err(&e),
        }
    }

    /// Every entity kind this build can address in a vault, in the order the
    /// docks list them (`cartalith_vault::EntityKind`) — so GDScript passes a
    /// string this engine actually parses rather than a transcribed literal.
    ///
    /// `"faction"` joined the list on 2026-08-25 (`GUI_GAP_REGISTER.md`
    /// **CV-22**), `"culture"` the same day (**CV-02**), and `"landmark"` on
    /// 2026-09-06 (owner ruling 13). `"poi"` is deliberately absent and
    /// always will be while this port has no point-of-interest entity
    /// (CV-01).
    ///
    /// **Nothing in `shell/` calls this yet.** Measured 2026-09-06:
    /// `grep -rn "vault_entity_kinds" godot-project/shell/*.gd` returns the
    /// `engine_bridge.gd` wrapper's own four lines (4034-4036, 4042) and
    /// two doc-comment mentions in `app.gd`, and no call site. So adding a
    /// kind here moves no pixel today; it states what the engine can
    /// address, which is the contract this method exists to hold. The
    /// fallback literal inside that wrapper — the one a build too old for
    /// this binding falls back to — still lists five and is now one short.
    #[func]
    fn vault_entity_kinds(&self) -> PackedStringArray {
        [
            EntityKind::Settlement,
            EntityKind::Province,
            EntityKind::Continent,
            EntityKind::Faction,
            EntityKind::Culture,
            EntityKind::Landmark,
            EntityKind::Label,
            EntityKind::Icon,
        ]
        .iter()
        .map(|k| GString::from(k.as_str()))
        .collect()
    }

    /// The `entity_id` to pass every `vault_*` call for the landmark whose
    /// `cartalith_civ::landmark::Landmark::key()` is `key` -- owner ruling
    /// 13's one new binding, and the only way GDScript can address a
    /// landmark at all, because `landmarks()` deliberately publishes
    /// `Landmark::id` (a position) and not this.
    ///
    /// `{ok, error, entity_id, label}`.
    ///
    /// **It resolves against the current run rather than just hashing the
    /// string.** `cartalith_vault::links::landmark_entity_id` is a total
    /// function -- it answers for `""` as readily as for
    /// `"waterfall@120,64"` -- so hashing alone would hand the caller a
    /// plausible id for a landmark that does not exist and let them file a
    /// note against nothing. A key with no landmark behind it is an `error`
    /// here, and `entity_id` is **absent** from the dictionary rather than
    /// present as a zero.
    ///
    /// `label` is the landmark's Ruling EI name -- "Waterfall near Vhal Serai",
    /// or "Waterfall (120, 64)" where no named place is near -- which is what
    /// `vault_attach`'s `entity_label` wants and what
    /// `cartalith_vault::template::suggested_path` turns into
    /// `Landmarks/Waterfall near Vhal Serai.md`. A landmark has no name of its own
    /// stored (see `export.rs`'s `EVERY`); the name is derived on each call
    /// (`cartalith_civ::landmark::place_name`).
    #[func]
    fn vault_landmark_entity_id(&self, key: GString) -> VarDictionary {
        let key = key.to_string();
        let Some(run) = self.landmark_store.last.as_ref() else {
            return err("No landmark pass has run for this world.");
        };
        let Some(_lm) = landmark_with_entity_id(
            &run.landmarks,
            cartalith_vault::links::landmark_entity_id(&key),
        )
        .filter(|lm| lm.key() == key) else {
            return err(format!("No landmark in this run has the key {key:?}."));
        };
        let mut d = ok();
        d.set("entity_id", cartalith_vault::links::landmark_entity_id(&key));
        // Ruling EI: the label is the landmark's generated (or user-given) name, which is also
        // what `[[...]]` links write; the type-and-cell designation is that name's own
        // fallback, so a world with no near place still gets the old label.
        if let Some(n) = self.landmark_name_list().into_iter().find(|n| n.key == key) {
            d.set("label", n.name.as_str());
        }
        d
    }

    /// Ruling EQ item 2: the `entity_id` to pass every `vault_*` call for the
    /// hand-placed label at `index` (the index `label_list` and the context
    /// hit carry), filed by its cell (`cartalith_vault::links::label_key`).
    /// `{ok, error, entity_id, label}`; `entity_id` is **absent** on a refusal
    /// (an index the list does not hold, or no labels yet), never `0`.
    /// `label` is the label's text, or `Label (x, y)` for an untitled one, which
    /// is what `vault_attach`'s `entity_label` and the suggested note path want.
    #[func]
    fn vault_label_entity_id(&self, index: i64) -> VarDictionary {
        let Some(lb) = usize::try_from(index).ok().and_then(|i| self.labels.as_ref()?.labels.get(i)) else {
            return err("No label at that index.");
        };
        let mut d = ok();
        d.set("entity_id", cartalith_vault::links::label_entity_id(lb.x, lb.y));
        let name = lb.name.trim();
        let text = if name.is_empty() { format!("Label ({}, {})", lb.x.round(), lb.y.round()) } else { name.to_string() };
        d.set("label", text.as_str());
        d
    }

    /// Ruling EQ item 2: as [`WorldGen::vault_label_entity_id`], for the placed
    /// icon at `index` (the index `icon_list` and the context hit carry). The
    /// label is `<Slot> icon (x, y)`: an icon has no name of its own.
    #[func]
    fn vault_icon_entity_id(&self, index: i64) -> VarDictionary {
        let Some(ic) = usize::try_from(index).ok().and_then(|i| self.icons.as_ref()?.icons.get(i)) else {
            return err("No icon at that index.");
        };
        let mut d = ok();
        d.set("entity_id", cartalith_vault::links::icon_entity_id(ic.x, ic.y));
        let text = format!("{} icon ({}, {})", capitalise(&ic.slot), ic.x.round(), ic.y.round());
        d.set("label", text.as_str());
        d
    }

    /// Ruling DP's private badge: which of `rels` are private, `{ok, marks, decided}`.
    ///
    /// `marks` is `{rel: label}` for each private note, the label the fixed-vocabulary words of
    /// `cartalith_vault::private::PrivateMarker::label` ("a folder named dm", "frontmatter
    /// private: true"), never note text. `decided` lists every `rel` whose answer is final. **A
    /// `rel` in neither is undecided**: it was past the read budget or could not be read, and a list
    /// must draw it as unknown rather than as public ("not looked at" is not "not private").
    ///
    /// A note in a private folder is decided from its path with no read; any other note is read
    /// once. `max_reads <= 0` means 400 reads, so a vault of thousands is badged over several calls
    /// (pass the leftovers back) rather than in one stall. Reads are of the note's text only, kept
    /// nowhere. The rule is `cartalith_vault::private::private_marks`, which has the tests.
    #[func]
    fn vault_private_marks(&self, rels: PackedStringArray, max_reads: i64) -> VarDictionary {
        let budget = if max_reads > 0 { max_reads as usize } else { 400 };
        let paths: Vec<String> = rels.as_slice().iter().map(|r| r.to_string()).collect();
        let got = cartalith_vault::private::private_marks(&paths, budget, |p| self.vault.read(p).ok());
        let mut marks = VarDictionary::new();
        for (p, m) in &got.private {
            marks.set(p.as_str(), m.label().as_str());
        }
        let decided: PackedStringArray = got.decided.iter().map(|p| GString::from(p.as_str())).collect();
        let mut d = ok();
        d.set("marks", &marks);
        d.set("decided", &decided);
        d
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

    /// Authored events for this entity (`STORY_PLANNING_SCOPE.md` SP-3,
    /// Ruling AM): every ` ```chronos ` block in its attached notes, via
    /// `cartalith_vault::VaultSession::entity_chronos`.
    ///
    /// `{ok, error, notes: [{rel, live, blocks, skipped}], events: [{rel,
    /// kind, start, end?, color?, group?, name, description?}]}`. `events`
    /// is merged across notes and sorted by `start` then `end` (stable, so
    /// same-year entries keep their written order) — SP-3's "interleaved by
    /// date". **Optional keys are absent, never defaulted**: a single-year
    /// event has no `end`, not `end == start`. `live` is false when a note
    /// was read from its saved copy because the vault is not reachable.
    /// Read-only; an unknown `kind` is an `error`.
    #[func]
    fn vault_entity_chronos(&self, kind: GString, entity_id: i64) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind {kind}")) };
        let per_note = self.vault.entity_chronos(k, entity_id);
        let mut notes: Array<VarDictionary> = Array::new();
        let mut all: Vec<(&str, &cartalith_vault::chronos::Event)> = Vec::new();
        for (rel, live, c) in &per_note {
            let skipped: PackedStringArray = c.skipped.iter().map(|s| GString::from(s.as_str())).collect();
            let mut nd = vdict! {
                "rel" => rel.as_str(),
                "live" => *live,
                "blocks" => c.blocks as i64,
            };
            nd.set("skipped", &skipped);
            notes.push(&nd);
            all.extend(c.events.iter().map(|e| (rel.as_str(), e)));
        }
        all.sort_by_key(|(_, e)| (e.start, e.end.unwrap_or(e.start)));
        let events: Array<VarDictionary> = all
            .into_iter()
            .map(|(rel, e)| {
                let mut d = vdict! {
                    "rel" => rel,
                    "kind" => e.kind.as_str(),
                    "start" => e.start,
                    "name" => e.name.as_str(),
                };
                if let Some(v) = e.end {
                    d.set("end", v);
                }
                if let Some(v) = &e.color {
                    d.set("color", v.as_str());
                }
                if let Some(v) = &e.group {
                    d.set("group", v.as_str());
                }
                if let Some(v) = &e.description {
                    d.set("description", v.as_str());
                }
                d
            })
            .collect();
        let mut d = ok();
        d.set("notes", &notes);
        d.set("events", &events);
        d
    }

    /// Writes one authored event into the note `rel` as a Chronos line
    /// (SP-3, Ruling AM) via `cartalith_vault::VaultSession::add_chronos_event`
    /// — appended to its ` ```chronos ` block, which is created if absent.
    ///
    /// `event` takes `vault_entity_chronos`'s own event keys, so a row read
    /// out can be written back in: `start` (int) and `name` required;
    /// `end` (int), `color`, `group`, `description` optional and **absent
    /// when not given** — an empty string is refused by the serializer, not
    /// read as "none". `kind` defaults to `"event"`, the only kind SP-3
    /// authors. `expect_hash` is `vault_read_file_for_edit(rel).hash`.
    ///
    /// `{ok, error, hash, conflict}`: `conflict` is true exactly when the note
    /// changed since that read (`Error::SourceChanged`) and nothing was
    /// written, so the UI can say so instead of a generic failure.
    #[func]
    fn vault_add_chronos_event(&mut self, rel: GString, expect_hash: GString, event: VarDictionary) -> VarDictionary {
        use cartalith_vault::chronos::{Event, Kind};
        let text = |k: &str| event.get(k).map(|v| v.to_string());
        let int = |k: &str| event.get(k).map(|v| v.try_to::<i64>().map_err(|_| format!("{k} must be a whole number")));
        let Some(start) = int("start") else { return err("an event needs a start year") };
        let start = match start { Ok(v) => v, Err(e) => return err(e) };
        let end = match int("end").transpose() { Ok(v) => v, Err(e) => return err(e) };
        let kind = match text("kind").as_deref() {
            None | Some("event") => Kind::Event,
            Some(other) => return err(format!("only plain events are authored here, not {other:?}")),
        };
        let e = Event {
            kind,
            start,
            end,
            // The UI authors year-only events (SP-3); month/day stay absent,
            // so the line it writes is byte-for-byte what it wrote before
            // `chronos::Event` carried them.
            start_md: None,
            end_md: None,
            color: text("color"),
            group: text("group"),
            name: text("name").unwrap_or_default(),
            description: text("description"),
        };
        match self.vault.add_chronos_event(&rel.to_string(), &e, &expect_hash.to_string()) {
            Ok(hash) => {
                let mut d = ok();
                d.set("hash", hash);
                d.set("conflict", false);
                d
            }
            Err(e) => {
                let conflict = matches!(e, cartalith_vault::Error::SourceChanged { .. });
                let mut d = err(e);
                d.set("conflict", conflict);
                d
            }
        }
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
    /// Ruling BF: the `.md` files under the folder Obsidian's Templates
    /// plugin is set to (`.obsidian/templates.json`), else Templater's
    /// `templates_folder`; only when neither is set, every `.md` with
    /// "template" in its path. [`Self::vault_template_source`] says which.
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

    /// Creates `rel` from `template_rel` with Obsidian's placeholders filled
    /// -- `{{title}}` is `rel`'s basename, `{{date}}`/`{{time}}` are the
    /// system's local date and time in the vault's formats -- and `name` in
    /// the owner's `{{…Name}}`/`[Name]` tokens. Templater's `<% %>` and every
    /// other prompt are left as written (`cartalith_vault::template::fill`).
    ///
    /// `{ok, path, text}` or `{ok: false, error}`. **Refuses an existing
    /// path** rather than overwriting it — the one thing that makes creating
    /// a note safe, where editing one needs a preview and a hash.
    ///
    /// Does not attach: attaching is its own validated act, and one button
    /// doing two writes is how a "create" quietly becomes an "overwrite".
    #[func]
    fn vault_create_from_template(&mut self, template_rel: GString, rel: GString, name: GString) -> VarDictionary {
        let Some(now) = local_now() else { return err("the system clock did not give a valid local date") };
        match self.vault.create_from_template(&template_rel.to_string(), &rel.to_string(), &name.to_string(), now) {
            Ok(text) => vdict! { "ok" => true, "path" => &rel, "text" => text.as_str() },
            Err(e) => err(e.to_string()),
        }
    }

    /// Where [`Self::vault_templates`] found its templates: `{ok, source,
    /// describe}` plus `folder` when a setting named one (the key is absent
    /// for the fallback, never an empty string). `source` is `obsidian`,
    /// `templater` or `fallback`.
    #[func]
    fn vault_template_source(&self) -> VarDictionary {
        match self.vault.template_config() {
            Ok(cfg) => {
                let mut d = vdict! { "ok" => true, "source" => cfg.source.as_str(), "describe" => cfg.describe().as_str() };
                if let Some(f) = cfg.folder.as_deref() {
                    d.set("folder", f);
                }
                d
            }
            Err(e) => err(e.to_string()),
        }
    }

    /// Obsidian's **Insert template**: `{ok, text, caret}` -- `note_text`
    /// with the filled template inserted at `caret` (a character offset) and
    /// its properties merged into the note's; `caret` is just after the
    /// inserted body. Writes nothing: the editor keeps the result until Save.
    #[func]
    fn vault_insert_template(&self, template_rel: GString, note_text: GString, caret: i64, title: GString) -> VarDictionary {
        let Some(now) = local_now() else { return err("the system clock did not give a valid local date") };
        match self.vault.insert_template(
            &template_rel.to_string(),
            &note_text.to_string(),
            caret.max(0) as usize,
            &title.to_string(),
            now,
        ) {
            Ok((text, c)) => vdict! { "ok" => true, "text" => text.as_str(), "caret" => c as i64 },
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

    /// Removes the link `link_id`; `true` if one was removed, `false` for an id
    /// no link carries. Touches only the link, never the note file it pointed at.
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
    ///
    /// **This is where a link whose `entity_kind` this build does not know is
    /// seen**, and it is why it is unfiltered. `SAVEFILE_COMPAT.md` §13.3.5
    /// requires such a link to be kept, *reported*, and written back
    /// unchanged; [`cartalith_vault::LinkKind`] does the keeping and the
    /// writing, and this list is the reporting — `entity_kind` carries the
    /// newer writer's own text (`"river"`, not a guess) beside the
    /// `entity_label` §13.3.5 says a person re-binds by. `vault_links_for`
    /// deliberately does **not** show it: that call is per-entity, and there
    /// is no entity.
    ///
    /// **One shell consequence, stated rather than left to be found.** The
    /// overview row is a button that calls `open_for(kind, …)`, and
    /// `vault_window.gd::_build_links` then asks `vault_links_for` for that
    /// kind, gets nothing back (correctly — the kind resolves to no entity),
    /// and takes its `links.is_empty()` branch: *"No notes attached to The
    /// Silverflow yet."* That sentence is **false** — the note is attached and
    /// is in the file — and it is the one place this fix leaves a wrong
    /// message rather than a missing one. Reported, not fixed: the shell is
    /// not this change's file. The data is safe either way; what is wrong is
    /// the wording of an empty state that has a cause it does not name.
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

    /// Writes into note `rel` the field fill that
    /// [`Self::vault_preview_field_fill`] previewed, for entity `entity_id` of
    /// `kind`. `expect_hash` is the preview's `hash`: the write is refused if
    /// the file changed since, so a person's edit between preview and write is
    /// never overwritten. `overwrite` picks `FieldFill::Overwrite` over
    /// `OnlyIfEmpty`. Returns `{ok, error, report}` (no `report` on a refusal),
    /// `report` as [`fill_report`] shapes it.
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
        let limit = if limit > 0 { limit as usize } else { cartalith_vault::LIST_CAP };
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

    /// Every note that links to `rel` — `{rel, form, count}`, `form` `"wiki"`
    /// or `"markdown"` — for a note being **browsed** rather than attached to
    /// an entity, where `vault_entity_backlinks` above has no `(kind,
    /// entity_id)` to look up.
    ///
    /// `vault_window.gd`'s own gap note named this exact absence
    /// (`_build_browse_preview`, 2026-09-23/26): `cartalith_vault::backlinks::
    /// Backlinks::backlinks_to` is path-keyed and already exists, and this is
    /// the one-line `#[func]` wrapper over it (`cartalith_vault::Vault::
    /// file_backlinks`) that was missing. Empty, not an error, when the
    /// backlink index has not been built — see `vault_backlink_stats().built`.
    #[func]
    fn vault_file_backlinks(&self, rel: GString) -> Array<VarDictionary> {
        self.vault
            .file_backlinks(&rel.to_string())
            .into_iter()
            .map(|(source, form, count)| {
                vdict! {
                    "rel" => source.as_str(),
                    "form" => match form {
                        cartalith_vault::LinkForm::Wiki => "wiki",
                        cartalith_vault::LinkForm::Markdown => "markdown",
                    },
                    "count" => count as i64,
                }
            })
            .collect()
    }

    /// Notes that name this file's own title in prose and do not link to it —
    /// `{rel, excerpt}`, the path-keyed analogue of `vault_entity_mentions`.
    /// The name searched for is the file's stem with `-`/`_` read as spaces
    /// (`cartalith_vault::Vault::file_mentions`'s own doc comment); there is
    /// no other title to search by for an unattached file. Empty under the
    /// same three conditions `vault_entity_mentions` names.
    #[func]
    fn vault_file_mentions(&self, rel: GString, max: i64) -> Array<VarDictionary> {
        let max = if max > 0 { max as usize } else { 12 };
        self.vault
            .file_mentions(&rel.to_string(), max)
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
    ///
    /// ## What "malformed" stopped meaning on 2026-09-06
    ///
    /// It used to include *"holds a link whose `entity_kind` I have never
    /// heard of"*. `LinkStore::from_json` failed the **whole document** over
    /// one such value — reproduced by doctoring one link of two from
    /// `"settlement"` to `"river"`: `unknown variant `river`, expected one of
    /// `settlement`, `province`, `continent`, `faction`, `culture`,
    /// `landmark``, and the valid link went with it. `SAVEFILE_COMPAT.md`
    /// §13.3.3 says the opposite in terms — *"An unrecognised value MUST NOT
    /// drop the link"* — and §13.3.5 says what to do instead: keep it, report
    /// it, write it back byte-equivalent. `cartalith_vault::LinkKind` keeps
    /// and writes; the `godot_warn!` below is the reporting, and
    /// [`Self::vault_all_links`] is where the user then sees the link itself.
    ///
    /// The warning names the kinds rather than only counting them, because
    /// the useful thing to know is *which* vocabulary this build is behind on
    /// — a count says a number, a name says which build wrote the file.
    #[func]
    fn vault_restore_state(&mut self, json: GString) -> bool {
        match cartalith_vault::LinkStore::from_json(&json.to_string()) {
            Ok(store) => {
                let unknown = unresolvable_kind_report(&store);
                self.vault.store = store;
                if let Some(report) = unknown {
                    godot_warn!("cartalith-godot: {report}");
                }
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

/// The note editor's Obsidian-compatible helpers (Ruling BF): link syntax,
/// the one link resolver, rename with link rewriting, `[[` suggestions,
/// tasks, tags, properties, headings, word count and unlinked mentions.
///
/// Thin wrappers. Every rule lives in `cartalith-vault`
/// (`notelinks`, `noteedit`, `frontmatter`, `suggest`) where it is unit
/// tested; this block only marshals. Two conventions hold throughout:
///
/// - **Offsets are characters** (Unicode code points, what a GDScript
///   `String` indexes by) and **lines are 0-based**.
/// - **An absent value is an absent key**, never `""`, `0` or `false` standing
///   in for it — read optional keys with `has()`. A failure is the file's
///   usual `{ok: false, error}`. Nothing here unwraps input-derived data.
///
/// Most of these take the editor's current text and touch no file; the two
/// rename calls and the two vault-backed lookups read the bound vault, and
/// only `vault_rename` writes.
#[godot_api(secondary)]
impl WorldGen {
    // -- link syntax --------------------------------------------------------

    /// Every link in `text`, in order, outside code: `[{start, end, line,
    /// syntax, external}]` plus `target`, `heading`, `block` and `alias` when
    /// written. `syntax` is `wiki`, `embed`, `markdown` or `markdown_embed`;
    /// `external` is true for a URL. A same-note link (`[[#Heading]]`) has no
    /// `target`. (`cartalith_vault::notelinks::link_spans`)
    #[func]
    fn vault_link_spans(&self, text: GString) -> Array<VarDictionary> {
        cartalith_vault::notelinks::link_spans(&text.to_string()).iter().map(span_dict).collect()
    }

    /// The link under a caret at character `offset`: `{found: true, …}` with
    /// `vault_link_spans`' keys, or `{found: false}` (also for a negative
    /// offset). A caret just past `]]` still finds its link.
    #[func]
    fn vault_link_at(&self, text: GString, offset: i64) -> VarDictionary {
        let found = usize::try_from(offset).ok().and_then(|o| cartalith_vault::notelinks::link_at(&text.to_string(), o));
        match found {
            Some(s) => {
                let mut d = span_dict(&s);
                d.set("found", true);
                d
            }
            None => vdict! { "found" => false },
        }
    }

    /// The links in `text` that name another note — no URLs, attachments or
    /// same-note links — in `vault_link_spans`' shape, in order and not
    /// deduplicated.
    #[func]
    fn vault_outgoing_links(&self, text: GString) -> Array<VarDictionary> {
        cartalith_vault::notelinks::outgoing_links(&text.to_string()).iter().map(span_dict).collect()
    }

    /// Which note `target`, written in the note at `source`, names — the
    /// resolver backlinks and broken links use, over the bound vault's notes.
    /// `{ok, status}` with `status` `unique` (plus `path`), `ambiguous` (plus
    /// `candidates`) or `none`; `{ok: false, error}` with no vault bound.
    #[func]
    fn vault_resolve_link(&self, source: GString, target: GString) -> VarDictionary {
        match self.vault.resolve_link(&source.to_string(), &target.to_string()) {
            Ok(r) => resolution_dict(&r),
            Err(e) => err(e),
        }
    }

    // -- rename -------------------------------------------------------------

    /// What renaming (or moving) the note `old` to `new` would do, computed
    /// and written nowhere: `{ok, from, to, hash, links, read, unreadable,
    /// knowledge_links, files: [{path, links}], ambiguous: [{path, line,
    /// target, candidates}]}`.
    ///
    /// `files` are the notes whose text changes, by their path before the
    /// rename; `links` totals the links rewritten; `ambiguous` lists links
    /// left alone because they could mean another note too; `unreadable`
    /// counts notes that could not be opened and so were not checked;
    /// `knowledge_links` counts entity links that will follow the note.
    /// `hash` must be handed to `vault_rename` unchanged. Refreshes the
    /// backlink index first (reading only notes that changed).
    #[func]
    fn vault_rename_plan(&mut self, old: GString, new: GString) -> VarDictionary {
        match self.vault.rename_preview(&old.to_string(), &new.to_string()) {
            Ok(p) => {
                let files: Array<VarDictionary> = p
                    .plan
                    .files
                    .iter()
                    .map(|f| vdict! { "path" => f.path.as_str(), "links" => f.links as i64 })
                    .collect();
                let ambiguous: Array<VarDictionary> = p
                    .plan
                    .ambiguous
                    .iter()
                    .map(|a| {
                        let mut d = vdict! { "path" => a.path.as_str(), "line" => a.line as i64, "target" => a.target.as_str() };
                        d.set("candidates", &packed(&a.candidates));
                        d
                    })
                    .collect();
                let mut d = ok();
                d.set("from", p.plan.from.as_str());
                d.set("to", p.plan.to.as_str());
                d.set("hash", p.hash.as_str());
                d.set("links", p.plan.link_count() as i64);
                d.set("read", p.read as i64);
                d.set("unreadable", p.unreadable as i64);
                d.set("knowledge_links", p.knowledge_links as i64);
                d.set("files", &files);
                d.set("ambiguous", &ambiguous);
                d
            }
            Err(e) => err(e),
        }
    }

    /// Renames (or moves) the note `old` to `new` and rewrites every link the
    /// plan named. `expect_hash` is `vault_rename_plan(old, new).hash`; if any
    /// note the plan would write changed since, nothing is written and the
    /// result is `{ok: false, error, conflict: true}` (re-plan and confirm
    /// again).
    ///
    /// The note is moved first, through the vault provider, so a refused move
    /// writes nothing at all; then each note's links are rewritten through
    /// the same re-read-and-compare guard as every write. `{ok: true, path,
    /// conflict: false, links, relinked, written: [{path, links}], failed:
    /// [{path, error}]}`: `ok` means the note moved; `failed` names any note
    /// whose links could not be updated afterwards (it changed during the
    /// rename, or could not be written) and still names the old path;
    /// `relinked` counts entity links re-pointed at the new path.
    #[func]
    fn vault_rename(&mut self, old: GString, new: GString, expect_hash: GString) -> VarDictionary {
        match self.vault.rename(&old.to_string(), &new.to_string(), &expect_hash.to_string()) {
            Ok(o) => {
                let written: Array<VarDictionary> =
                    o.written.iter().map(|(p, n)| vdict! { "path" => p.as_str(), "links" => *n as i64 }).collect();
                let failed: Array<VarDictionary> =
                    o.failed.iter().map(|(p, e)| vdict! { "path" => p.as_str(), "error" => e.as_str() }).collect();
                let mut d = ok();
                d.set("path", o.to.as_str());
                d.set("conflict", false);
                d.set("links", o.written.iter().map(|(_, n)| *n as i64).sum::<i64>());
                d.set("relinked", o.relinked as i64);
                d.set("written", &written);
                d.set("failed", &failed);
                d
            }
            Err(e) => {
                let conflict = matches!(e, cartalith_vault::Error::SourceChanged { .. });
                let mut d = err(e);
                d.set("conflict", conflict);
                d
            }
        }
    }

    // -- `[[` suggestions ---------------------------------------------------

    /// Ranks the caller's own candidates against `query` (exact, prefix, word
    /// start, substring, subsequence; ties alphabetical), at most `limit`
    /// (non-positive: 50). Each candidate is a Dictionary with a `name`
    /// String and optional `path`, `kind` and `aliases` (an Array or
    /// PackedStringArray of Strings); one without a `name` is skipped.
    ///
    /// `[{index, name, match, matched}]` plus `path` and `kind` when the
    /// candidate had them and `alias` when an alias matched. `index` is the
    /// candidate's position in `candidates`; `match` is the class;
    /// `matched` is the text that matched (show it).
    #[func]
    fn vault_suggest(&self, query: GString, candidates: VarArray, limit: i64) -> Array<VarDictionary> {
        let mut cands: Vec<cartalith_vault::suggest::Candidate> = Vec::new();
        let mut given: Vec<(usize, bool, bool)> = Vec::new(); // (caller's index, had path, had kind)
        for (i, v) in candidates.iter_shared().enumerate() {
            let Ok(d) = v.try_to::<VarDictionary>() else { continue };
            let text = |k: &str| d.get(k).and_then(|v| v.try_to::<GString>().ok()).map(|g| g.to_string());
            let Some(name) = text("name") else { continue };
            let (path, kind) = (text("path"), text("kind"));
            given.push((i, path.is_some(), kind.is_some()));
            cands.push(cartalith_vault::suggest::Candidate {
                name,
                path: path.unwrap_or_default(),
                kind: kind.unwrap_or_default(),
                aliases: d.get("aliases").map(|v| strings_of(&v)).unwrap_or_default(),
            });
        }
        let limit = if limit > 0 { limit as usize } else { 50 };
        cartalith_vault::suggest::suggest(&query.to_string(), &cands, limit)
            .into_iter()
            .filter_map(|s| {
                let c = cands.get(s.index)?;
                let &(index, had_path, had_kind) = given.get(s.index)?;
                let mut d = suggestion_dict(&s, c);
                d.set("index", index as i64);
                // Echo only what the caller gave: an absent key stays absent.
                if !had_path {
                    let _ = d.remove("path");
                }
                if !had_kind {
                    let _ = d.remove("kind");
                }
                Some(d)
            })
            .collect()
    }

    /// `vault_suggest` over the bound vault's notes (name = file name without
    /// `.md`, `kind` = `note`, aliases from the backlink index — so aliases
    /// appear only once the index has been built): `[{name, path, kind,
    /// match, matched}]` plus `alias` when an alias matched. Lists the vault
    /// once per call (no file is opened). Empty with no vault bound.
    #[func]
    fn vault_suggest_notes(&self, query: GString, limit: i64) -> Array<VarDictionary> {
        let Ok(cands) = self.vault.note_candidates() else { return Array::new() };
        let limit = if limit > 0 { limit as usize } else { 50 };
        cartalith_vault::suggest::suggest(&query.to_string(), &cands, limit)
            .into_iter()
            .filter_map(|s| cands.get(s.index).map(|c| suggestion_dict(&s, c)))
            .collect()
    }

    // -- Markdown helpers ---------------------------------------------------

    /// The headings in `text` for `[[Note#…`: `[{level, text, line, start,
    /// end}]`, fenced code and frontmatter skipped (the same headings
    /// `vault_attach` accepts).
    #[func]
    fn vault_note_headings(&self, text: GString) -> Array<VarDictionary> {
        cartalith_vault::noteedit::headings(&text.to_string())
            .into_iter()
            .map(|h| {
                vdict! {
                    "level" => h.level as i64,
                    "text" => h.text.as_str(),
                    "line" => h.line as i64,
                    "start" => h.start as i64,
                    "end" => h.end as i64,
                }
            })
            .collect()
    }

    /// Ticks or unticks the task on 0-based `line`, as clicking its box in
    /// Obsidian does: `{ok, text, checked}`, or `{ok: false, error}` when the
    /// line is not a task (or is inside code or the frontmatter). Writes
    /// nothing; save through `vault_write_file`.
    #[func]
    fn vault_toggle_task(&self, text: GString, line: i64) -> VarDictionary {
        let Ok(n) = usize::try_from(line) else { return err("a line number cannot be negative") };
        match cartalith_vault::noteedit::toggle_task(&text.to_string(), n) {
            Some(t) => {
                let mut d = ok();
                d.set("text", t.text.as_str());
                d.set("checked", t.checked);
                d
            }
            None => err(format!("line {line} is not a task")),
        }
    }

    /// What the embed whose inner text is `inner` (the text between `![[` and `]]`, or a Markdown
    /// image's address) turns into when drawn in the note `from` inside the chain `stack` (VE-7;
    /// `cartalith_vault::embed`). `stack` is the notes already being drawn, the open note first.
    ///
    /// `{ok, kind, ...}` where `kind` is `note` (`path`, `text` the excerpt, `truncated`, `heading`),
    /// `image` (`path`, `stamp`, `width` or absent), `missing`, `ambiguous` (`candidates`), `cycle`
    /// (`path`, `chain`), `toodeep` (`path`, `limit`), `fragment` (`path`, `fragment`), `escape`,
    /// `unsupported` (`why`) or `toolarge` (`path`, `bytes`, `limit`); each also carries `target`
    /// where it has one. `{ok: false, error}` with no vault bound. Reads, writes nothing.
    #[func]
    fn vault_embed_open(&self, from: GString, inner: GString, stack: PackedStringArray) -> VarDictionary {
        use cartalith_vault::embed::Embed;
        let stack: Vec<String> = stack.as_slice().iter().map(|s| s.to_string()).collect();
        let e = match self.vault.embed_open(&from.to_string(), &inner.to_string(), &stack) {
            Ok(e) => e,
            Err(e) => return err(e),
        };
        let mut d = ok();
        match e {
            Embed::Note { path, excerpt } => {
                d.set("kind", "note");
                d.set("path", path);
                d.set("text", excerpt.text);
                d.set("truncated", excerpt.truncated);
                if let Some(h) = excerpt.heading {
                    d.set("heading", h);
                }
            }
            Embed::Image { path, stamp, width } => {
                d.set("kind", "image");
                d.set("path", path);
                d.set("stamp", stamp);
                if let Some(w) = width {
                    d.set("width", w as i64);
                }
            }
            Embed::Missing { target } => {
                d.set("kind", "missing");
                d.set("target", target);
            }
            Embed::Ambiguous { target, candidates } => {
                d.set("kind", "ambiguous");
                d.set("target", target);
                let c: PackedStringArray = candidates.iter().map(|s| GString::from(s.as_str())).collect();
                d.set("candidates", &c);
            }
            Embed::Cycle { path, chain } => {
                d.set("kind", "cycle");
                d.set("path", path);
                let c: PackedStringArray = chain.iter().map(|s| GString::from(s.as_str())).collect();
                d.set("chain", &c);
            }
            Embed::TooDeep { path, limit } => {
                d.set("kind", "toodeep");
                d.set("path", path);
                d.set("limit", limit as i64);
            }
            Embed::FragmentMissing { path, fragment } => {
                d.set("kind", "fragment");
                d.set("path", path);
                d.set("fragment", fragment);
            }
            Embed::Escape { target } => {
                d.set("kind", "escape");
                d.set("target", target);
            }
            Embed::Unsupported { target, why } => {
                d.set("kind", "unsupported");
                d.set("target", target);
                d.set("why", why);
            }
            Embed::TooLarge { path, bytes, limit } => {
                d.set("kind", "toolarge");
                d.set("path", path);
                d.set("bytes", bytes as i64);
                d.set("limit", limit as i64);
            }
        }
        d
    }

    /// The bytes of an image inside the vault, for Preview (`rel` as `vault_embed_open` returned it;
    /// checked again for a safe path, a drawable extension, containment under the vault root and the
    /// size cap). Empty on any refusal: a caller reads emptiness as "not drawn" and has already been
    /// told why by `vault_embed_open`.
    #[func]
    fn vault_read_image(&self, rel: GString) -> PackedByteArray {
        match self.vault.read_image(&rel.to_string()) {
            Ok(b) => PackedByteArray::from(b.as_slice()),
            Err(_) => PackedByteArray::new(),
        }
    }

    /// The Chronos events in the ` ```chronos ` blocks of `text` (`cartalith_vault::chronos::parse`,
    /// pure): `{ok, blocks, events: [{kind, start, end?, name, color?, group?, description?}],
    /// skipped: [line]}`. `events` is in source order; a line that did not parse is in `skipped`, never
    /// dropped. Writes nothing.
    #[func]
    fn vault_note_chronos(&self, text: GString) -> VarDictionary {
        let c = cartalith_vault::chronos::parse(&text.to_string());
        let events: Array<VarDictionary> = c
            .events
            .iter()
            .map(|e| {
                let mut d = vdict! { "kind" => e.kind.as_str(), "start" => e.start, "name" => e.name.as_str() };
                if let Some(v) = e.end {
                    d.set("end", v);
                }
                if let Some(v) = &e.color {
                    d.set("color", v.as_str());
                }
                if let Some(v) = &e.group {
                    d.set("group", v.as_str());
                }
                if let Some(v) = &e.description {
                    d.set("description", v.as_str());
                }
                d
            })
            .collect();
        let skipped: PackedStringArray = c.skipped.iter().map(|s| GString::from(s.as_str())).collect();
        let mut d = ok();
        d.set("blocks", c.blocks as i64);
        d.set("events", &events);
        d.set("skipped", &skipped);
        d
    }

    /// Every tag in `text`: `[{tag, start, end, line, frontmatter}]`, `tag`
    /// without its `#`, `frontmatter` true for the `tags` property's values.
    /// Code, links, headings' own `#` and URL fragments are not tags.
    #[func]
    fn vault_note_tags(&self, text: GString) -> Array<VarDictionary> {
        cartalith_vault::noteedit::tags(&text.to_string())
            .into_iter()
            .map(|t| {
                vdict! {
                    "tag" => t.tag.as_str(),
                    "start" => t.start as i64,
                    "end" => t.end as i64,
                    "line" => t.line as i64,
                    "frontmatter" => t.frontmatter,
                }
            })
            .collect()
    }

    /// The note's word count, frontmatter excluded
    /// (`cartalith_vault::noteedit::word_count` says what a word is).
    #[func]
    fn vault_word_count(&self, text: GString) -> i64 {
        i64::try_from(cartalith_vault::noteedit::word_count(&text.to_string())).unwrap_or(i64::MAX)
    }

    /// The note's aliases from its `aliases`/`alias` property, in every form
    /// Obsidian reads, duplicates dropped.
    #[func]
    fn vault_note_aliases(&self, text: GString) -> PackedStringArray {
        packed(&cartalith_vault::frontmatter::aliases(&text.to_string()))
    }

    /// The note's properties for the Properties form (VE-5): `{present, editable, properties: [{key,
    /// list, values}]}`. `present` is false when the note has no frontmatter block (then `properties` is
    /// empty and `editable` is true: setting a key creates the block). `editable` is false when the block
    /// is malformed or repeats a key: the form lists what it could read and offers no edit.
    /// `cartalith_vault::frontmatter::properties` says how a scalar and a list are read.
    #[func]
    fn vault_note_properties(&self, text: GString) -> VarDictionary {
        let mut d = VarDictionary::new();
        match cartalith_vault::frontmatter::properties(&text.to_string()) {
            None => {
                d.set("present", false);
                d.set("editable", true);
                d.set("properties", &Array::<VarDictionary>::new());
            }
            Some((props, editable)) => {
                d.set("present", true);
                d.set("editable", editable);
                let rows: Array<VarDictionary> = props
                    .into_iter()
                    .map(|p| {
                        let mut row = vdict! { "key" => p.key.as_str(), "list" => p.list };
                        row.set("values", &packed(&p.values));
                        row
                    })
                    .collect();
                d.set("properties", &rows);
            }
        }
        d
    }

    /// Every tag in the vault with the notes that carry it (VE-5's Tags index):
    /// `{ok, tags: [{tag, notes: [rel]}], scanned, total}`, most-used first. Opens at most `max_reads`
    /// notes (`<= 0` means 2 000), so `scanned < total` means the index is partial and the caller says
    /// so. `{ok: false, error}` with no vault bound.
    #[func]
    fn vault_tag_index(&self, max_reads: i64) -> VarDictionary {
        let cap = if max_reads > 0 { max_reads as usize } else { 2000 };
        match self.vault.tag_index(cap) {
            Ok(ix) => {
                let mut d = ok();
                let rows: Array<VarDictionary> = ix
                    .tags
                    .iter()
                    .map(|t| {
                        let mut row = vdict! { "tag" => t.tag.as_str() };
                        row.set("notes", &packed(&t.notes));
                        row
                    })
                    .collect();
                d.set("tags", &rows);
                d.set("scanned", ix.scanned as i64);
                d.set("total", ix.total as i64);
                d
            }
            Err(e) => err(e),
        }
    }

    /// Sets one property, as Obsidian's Properties panel does, leaving every
    /// other line byte-identical: `{ok, text}` or `{ok: false, error}`.
    /// `value` may be a String (written as one scalar; `null` writes an empty
    /// value) or an Array/PackedStringArray (written as a list). Writes
    /// nothing; save through `vault_write_file`.
    #[func]
    fn vault_frontmatter_set(&self, text: GString, key: GString, value: Variant) -> VarDictionary {
        let v = frontmatter_value(&value);
        match cartalith_vault::frontmatter::frontmatter_set(&text.to_string(), &key.to_string(), &v) {
            Ok(t) => {
                let mut d = ok();
                d.set("text", t.as_str());
                d
            }
            Err(e) => err(e),
        }
    }

    /// Removes one property (case-insensitive key): `{ok, removed: true,
    /// text}`, or `{ok, removed: false}` with no `text` when the note has no
    /// such property, or `{ok: false, error}`. Writes nothing.
    #[func]
    fn vault_frontmatter_remove(&self, text: GString, key: GString) -> VarDictionary {
        match cartalith_vault::frontmatter::frontmatter_remove(&text.to_string(), &key.to_string()) {
            Ok(Some(t)) => {
                let mut d = ok();
                d.set("removed", true);
                d.set("text", t.as_str());
                d
            }
            Ok(None) => {
                let mut d = ok();
                d.set("removed", false);
                d
            }
            Err(e) => err(e),
        }
    }

    // -- unlinked mentions --------------------------------------------------

    /// Every unlinked mention of `name` in `text`: whole-word,
    /// case-insensitive, outside links, code, the frontmatter and URLs —
    /// `[{start, end, line, text}]`, `text` as written there.
    #[func]
    fn vault_mention_spans(&self, text: GString, name: GString) -> Array<VarDictionary> {
        cartalith_vault::noteedit::mention_spans(&text.to_string(), &name.to_string())
            .into_iter()
            .map(|m| {
                vdict! {
                    "start" => m.start as i64,
                    "end" => m.end as i64,
                    "line" => m.line as i64,
                    "text" => m.text.as_str(),
                }
            })
            .collect()
    }

    /// **Link it**: replaces characters `start..end` with `[[target]]`, or
    /// `[[target|as written]]` when the text there differs from `target`:
    /// `{ok, text, caret}` (`caret` just after the link) or `{ok: false,
    /// error}` when the span is out of range, crosses a line, or touches code,
    /// a link or the frontmatter. Writes nothing; save through
    /// `vault_write_file` with the hash the text was read at.
    #[func]
    fn vault_link_mention(&self, text: GString, start: i64, end: i64, target: GString) -> VarDictionary {
        let (Ok(s), Ok(e)) = (usize::try_from(start), usize::try_from(end)) else {
            return err("an offset cannot be negative");
        };
        match cartalith_vault::noteedit::link_mention(&text.to_string(), s, e, &target.to_string()) {
            Some((t, caret)) => {
                let mut d = ok();
                d.set("text", t.as_str());
                d.set("caret", caret as i64);
                d
            }
            None => err("that text cannot be linked here: it is out of range, crosses a line, or touches code, a link or the properties"),
        }
    }
}

/// A [`cartalith_vault::notelinks::LinkSpan`] as the editor's dictionary:
/// `{start, end, line, syntax, external}`, plus `target`, `heading`, `block`
/// and `alias` only when the author wrote them — absent, never `""`.
fn span_dict(s: &cartalith_vault::notelinks::LinkSpan) -> VarDictionary {
    let mut d = vdict! {
        "start" => s.start as i64,
        "end" => s.end as i64,
        "line" => s.line as i64,
        "syntax" => s.syntax.as_str(),
        "external" => s.external,
    };
    for (key, value) in [("target", &s.target), ("heading", &s.heading), ("block", &s.block), ("alias", &s.alias)] {
        if let Some(v) = value {
            d.set(key, v.as_str());
        }
    }
    d
}

/// A [`cartalith_vault::notelinks::Resolution`] as `{ok, status}` plus
/// `path` (unique) or `candidates` (ambiguous); `none` carries neither.
fn resolution_dict(r: &cartalith_vault::notelinks::Resolution) -> VarDictionary {
    use cartalith_vault::notelinks::Resolution;
    let mut d = ok();
    match r {
        Resolution::Unique(p) => {
            d.set("status", "unique");
            d.set("path", p.as_str());
        }
        Resolution::Ambiguous(c) => {
            d.set("status", "ambiguous");
            d.set("candidates", &packed(c));
        }
        Resolution::Unresolved => d.set("status", "none"),
    }
    d
}

/// One ranked suggestion: `{name, path, kind, match, matched}` plus `alias`
/// when an alias matched.
fn suggestion_dict(s: &cartalith_vault::suggest::Suggestion, c: &cartalith_vault::suggest::Candidate) -> VarDictionary {
    let mut d = vdict! {
        "name" => c.name.as_str(),
        "path" => c.path.as_str(),
        "kind" => c.kind.as_str(),
        "match" => s.kind.as_str(),
        "matched" => s.matched.as_str(),
    };
    if let Some(a) = &s.alias {
        d.set("alias", a.as_str());
    }
    d
}

/// Strings as a `PackedStringArray`.
fn packed(v: &[String]) -> PackedStringArray {
    v.iter().map(|s| GString::from(s.as_str())).collect()
}

/// The Strings in a `PackedStringArray`, an `Array` (its String elements;
/// anything else is skipped, not stringified) or a single String.
fn strings_of(v: &Variant) -> Vec<String> {
    if let Ok(p) = v.try_to::<PackedStringArray>() {
        return p.as_slice().iter().map(|g| g.to_string()).collect();
    }
    if let Ok(a) = v.try_to::<VarArray>() {
        return a.iter_shared().filter_map(|x| x.try_to::<GString>().ok()).map(|g| g.to_string()).collect();
    }
    v.try_to::<GString>().map(|g| vec![g.to_string()]).unwrap_or_default()
}

/// A GDScript value as a property value: an Array or PackedStringArray is a
/// list (each element as text), `null` is an empty value (YAML's null — no
/// value, not a made-up one), anything else its text.
fn frontmatter_value(v: &Variant) -> cartalith_vault::frontmatter::FrontmatterValue {
    use cartalith_vault::frontmatter::FrontmatterValue;
    // Dispatched on the Variant's TYPE, not by trying conversions: `try_to::<PackedStringArray>()` on a
    // String Variant SUCCEEDS with an empty array, so the old try-in-order version wrote every scalar
    // as `key: []` (found by `_ve5_probe.gd`'s first property edit, 2026-11-08).
    match v.get_type() {
        VariantType::NIL => FrontmatterValue::Text(String::new()),
        VariantType::PACKED_STRING_ARRAY => {
            let p = v.to::<PackedStringArray>();
            FrontmatterValue::List(p.as_slice().iter().map(|g| g.to_string()).collect())
        }
        VariantType::ARRAY => {
            let a = v.to::<VarArray>();
            FrontmatterValue::List(a.iter_shared().map(|x| x.to_string()).collect())
        }
        _ => FrontmatterValue::Text(v.to_string()),
    }
}

/// Sort key for a link's status where the *loudest* state must win: lower is
/// louder (`Unbound` 0 ... `Connected` 5), so `min_by_key` finds the state a
/// person most needs to see, and `Connected` is reported only when nothing else
/// applies. The order is a judgement made for the panel badge, not a lifecycle.
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

/// A fill result as the `report` array the panel lists: one `{field, outcome}`
/// per field, `outcome` being `written`, `skipped_occupied` (the field already
/// had a value and the policy was `OnlyIfEmpty`) or `not_found` (the note has no
/// such field).
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
    ///
    /// **Two kinds are answered before the `civ` guard and are not subject to
    /// that first clause.** A culture is seven compile-time rows and is real
    /// with no world at all; a landmark (owner ruling 13, 2026-09-06) lives
    /// in `self.landmark_store`, which the landmark pass fills without ever
    /// reading `CivData` — so both would have returned empty for reasons that
    /// are not true of them.
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

        // Owner ruling 13, 2026-09-06. Answered before the `civ` guard for
        // the same reason a culture is, and a different one: the landmark
        // pass reads `WorldSource::Generated` and never touches `CivData`,
        // so a world with landmarks and no civ layer is an ordinary state
        // rather than an edge case, and gating this on `self.civ` would show
        // an empty panel for every one of them.
        //
        // **Resolution is by `Landmark::key()`, walked, never by index** --
        // see [`landmark_with_entity_id`]. A landmark that is not in the
        // current run answers an empty map, which is `entity_values`' own
        // documented "an entity that has gone away is not an error here".
        // Ruling EQ item 2: a label or an icon answers its type and its cell,
        // before the `civ` guard for the landmark's reason (neither needs a
        // civilisation layer). One that is no longer placed answers an empty map.
        if kind == EntityKind::Label || kind == EntityKind::Icon {
            let Some((x, y)) = self.item_cell_for_entity(kind, entity_id) else { return out };
            out.insert("entity_type", kind.as_str().to_string());
            out.insert("coordinates", format!("{}, {}", x.round(), y.round()));
            self.append_snapshot_fields(&mut out, kind, entity_id);
            return out;
        }
        if kind == EntityKind::Landmark {
            let Some(lm) = self.landmark_for_entity(entity_id) else { return out };
            out.insert("entity_type", kind.as_str().to_string());
            // `LandmarkKindSpec::label`, resolved rather than title-cased
            // from the key: "river_confluence" is "River confluence" and
            // `capitalise` would render it "River_confluence". A kind that
            // does not resolve costs the row -- the same rule `LandmarkDto`
            // applies on load -- rather than showing a machine token.
            if let Some(spec) = cartalith_civ::landmark::kind_spec(&lm.kind) {
                out.insert("landmark_type", spec.label.to_string());
            }
            out.insert("coordinates", format!("{}, {}", lm.x, lm.y));
            // `Landmark::elevation` is **metres above sea level** and says so
            // in its own doc; `elevation_m` names the unit for exactly that
            // reason, and is a different registry key from the settlement's
            // normalised `elevation`. A non-finite value is omitted rather
            // than printed: "we did not measure this" is not a number.
            if lm.elevation.is_finite() {
                out.insert("elevation_m", format!("{} m", thousands(lm.elevation.round() as i64)));
            }
            // Research 22's `causal_chain`, joined cause-first with the
            // arrow its own doc comment uses ("ordered cause -> consequence
            // -> landmark"). Empty stays absent -- an empty chain is a run
            // that recorded none, not a landmark with no cause.
            if !lm.causal.is_empty() {
                out.insert("causal", lm.causal.join(" \u{2192} "));
            }
            self.append_snapshot_fields(&mut out, kind, entity_id);
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
                    // R1: a custom religion's own name, or "Missing religion
                    // (custom:N)" for a dangling one -- never its raw key and
                    // never a guessed built-in.
                    let label = if e.religion.starts_with(cartalith_civ::religion_library::CUSTOM_RELIGION_PREFIX) {
                        civ.religions.display_name(&e.religion)
                    } else {
                        self.vocab_label(&cartalith_civ::roster::CIV_RELIGIONS, &e.religion)
                    };
                    out.insert("religion", label);
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
            // Answered above, before the `civ` guard, for the same reason
            // `Culture` is: a landmark run needs `WorldSource::Generated`
            // and no civ layer at all, so gating it on `self.civ` would have
            // hidden every landmark from a user who has never pressed
            // Recompute. Same empty-arm reasoning as the line above.
            EntityKind::Landmark => {}
            // Answered above, before the `civ` guard (Ruling EQ item 2).
            EntityKind::Label | EntityKind::Icon => {}
        }
        self.append_snapshot_fields(&mut out, kind, entity_id);
        out
    }

    /// §19's Map group appended to `out` — the one set of values that is not
    /// read out of `CivData` at all, because a snapshot is a file somebody
    /// generated rather than a property of the world. Filed on the link store
    /// by `vault_snapshot`, and absent until one exists, which is what keeps
    /// `export::offer` from putting a Map checkbox in front of a user with
    /// no image behind it.
    ///
    /// The value is the Markdown image `export.rs`'s module doc specifies:
    /// a relative path, never a base64 payload (§22).
    ///
    /// **A filed path is a record that a snapshot was written, not proof
    /// it is still there** (2026-09-05). The store is the only thing this
    /// loop used to consult, so deleting or moving `.cartalith/maps/*.png`
    /// from outside Cartalith left the Map checkbox offered and let
    /// `vault_block_body` write `![](…)` into the user's own note pointing
    /// at nothing — §20's "must not expose information that the entity
    /// does not possess" failing on the one field whose value lives
    /// outside the process. `snapshot_on_disk` closes it, and only ever
    /// *removes* a field: `None` (this device cannot check) keeps the old
    /// behaviour rather than hiding a snapshot nobody verified was gone.
    ///
    /// Extracted from the tail of [`WorldGen::entity_values`] on 2026-09-06,
    /// unchanged, because owner ruling 13's landmark arm answers **before**
    /// the `civ` guard and still has to reach it: a landmark is in
    /// `export::PLACED`, so it is offered the three Map fields exactly as a
    /// settlement is.
    fn append_snapshot_fields(
        &self,
        out: &mut BTreeMap<&'static str, String>,
        kind: EntityKind,
        entity_id: i64,
    ) {
        let key = entity_key(kind, entity_id);
        for (field, radius, _) in export::MAP_RADII {
            if let Some(rel) = self.vault.store.snapshot(&key, radius) {
                if self.snapshot_on_disk(rel) == Some(false) {
                    continue;
                }
                out.insert(*field, format!("![]({rel})"));
            }
        }
    }

    /// The landmark in the **current run** that `entity_id` addresses, or
    /// `None`.
    ///
    /// `None` is the ordinary state, not a failure: `LandmarkStore::last` is
    /// dropped by `LandmarkStore::invalidate()`, which `WorldGen::absorb`,
    /// `center_landmasses` and `load_save` all call (measured 2026-09-06;
    /// `generate`/`generate_sized` reach it through `absorb`) — so between a
    /// regenerate and the next landmark pass every landmark link is
    /// unresolved. `project_open` is the exception and restores `last` from
    /// the archive's own `LandmarksDoc.results`. `SAVEFILE_COMPAT.md` 13.3.5's rule
    /// applies -- keep the link, report it, write it back unchanged -- and
    /// nothing here removes one.
    fn landmark_for_entity(&self, entity_id: i64) -> Option<&cartalith_civ::landmark::Landmark> {
        landmark_with_entity_id(&self.landmark_store.last.as_ref()?.landmarks, entity_id)
    }

    /// Ruling EQ item 2: the cell `(x, y)` of the hand-placed label or placed
    /// icon `entity_id` addresses, or `None` when it is no longer placed or
    /// the id is ambiguous (two items in one cell: refused, like a landmark
    /// collision, so a note is never attached to the wrong one).
    fn item_cell_for_entity(&self, kind: EntityKind, entity_id: i64) -> Option<(f64, f64)> {
        match kind {
            EntityKind::Label => label_cell(&self.labels.as_ref()?.labels, entity_id),
            EntityKind::Icon => icon_cell(&self.icons.as_ref()?.icons, entity_id),
            _ => None,
        }
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
    /// The lookups mirror [`WorldGen::entity_values`]' own arms — a
    /// settlement by `tid`, a province by its capital, a continent by its
    /// centroid, a faction by its seat of power, a **landmark by its own
    /// cell** — deliberately rather than parsing the `coordinates` string
    /// that function formats. A snapshot centred on a re-parsed display
    /// string would be one rounding decision away from a different cell.
    /// (This paragraph said "the three lookups" while there were four, and
    /// is now written against the match rather than counted.)
    pub(crate) fn entity_cell(&self, kind: EntityKind, entity_id: i64) -> Option<(i64, i64)> {
        let (gw, gh) = (self.gw.max(0) as i64, self.gh.max(0) as i64);
        let (x, y) = match kind {
            // Owner ruling 13. **The only arm that does not consult `civ`**,
            // which is why the `let civ = self.civ.as_ref()?` that used to
            // stand above this match now lives inside the four arms that
            // need it: a landmark run needs no civ layer, and hoisting the
            // guard would have made every landmark unsnapshottable until
            // someone pressed Recompute.
            //
            // A landmark is the most literally placed kind here -- `x` and
            // `y` *are* the cell, with no capital, centroid or seed in
            // between.
            EntityKind::Landmark => {
                landmark_cell(&self.landmark_store.last.as_ref()?.landmarks, entity_id)?
            }
            EntityKind::Settlement => {
                let civ = self.civ.as_ref()?;
                let s = civ.settlements.iter().find(|s| s.tid as i64 == entity_id)?;
                (s.placement.x as i64, s.placement.y as i64)
            }
            EntityKind::Province => {
                let civ = self.civ.as_ref()?;
                let p = civ.province_list.iter().find(|p| p.id as i64 == entity_id)?;
                let cap = civ.settlements.get(p.capital_settlement_index)?;
                (cap.placement.x as i64, cap.placement.y as i64)
            }
            EntityKind::Continent => {
                let civ = self.civ.as_ref()?;
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
                let civ = self.civ.as_ref()?;
                if entity_id < 1 || entity_id as usize >= civ.faction_roster.0.len() {
                    return None;
                }
                let fid = entity_id as i32;
                let cap = civ.settlements.iter().find(|s| s.placement.faction == fid && s.placement.capital)?;
                (cap.placement.x as i64, cap.placement.y as i64)
            }
            EntityKind::Culture => return None,
            // Ruling EQ item 2: the item's own cell, floored (a label or icon sits at a fractional grid position).
            EntityKind::Label | EntityKind::Icon => {
                let (fx, fy) = self.item_cell_for_entity(kind, entity_id)?;
                (fx.floor() as i64, fy.floor() as i64)
            }
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

/// The landmark in `landmarks` whose `key()` hashes to `entity_id` --
/// owner ruling 13's whole resolution path, in one place and free of
/// `WorldGen` so it can be tested without a Godot engine.
///
/// **It walks and recomputes; it never indexes.** That is the property the
/// ruling turns on: `Landmark::id` is "a position in
/// `LandmarkResult::landmarks`" that moves when a cap moves or a kind is
/// disarmed, while `key()` is `"<kind>@<x>,<y>"` and moves only when the
/// landmark itself does. A re-run that places different landmarks around this
/// one leaves this one's key -- and therefore this lookup -- alone.
///
/// **An ambiguous id resolves to nothing.** `cartalith_vault::links::
/// landmark_entity_id` is 52 bits, so a collision is possible (around 5e-12
/// for a few hundred landmarks) and "unlikely" is not "cannot": taking the
/// first match would silently attach someone's note to the wrong feature,
/// while `None` makes it an unresolved link they can re-bind. Two rows with
/// the same key -- which `the_stable_key_is_unique_within_a_run` says a real
/// run never produces -- take the same branch.
pub(crate) fn landmark_with_entity_id(
    landmarks: &[cartalith_civ::landmark::Landmark],
    entity_id: i64,
) -> Option<&cartalith_civ::landmark::Landmark> {
    let mut hits = landmarks
        .iter()
        .filter(|lm| cartalith_vault::links::landmark_entity_id(&lm.key()) == entity_id);
    let first = hits.next()?;
    hits.next().is_none().then_some(first)
}

/// Ruling EQ item 2: `(x, y)` of the hand-placed label `entity_id` addresses
/// (`cartalith_vault::links::label_entity_id`), refusing an ambiguous id.
pub(crate) fn label_cell(labels: &[cartalith_civ::labels::MapLabel], entity_id: i64) -> Option<(f64, f64)> {
    let mut hits = labels.iter().filter(|lb| cartalith_vault::links::label_entity_id(lb.x, lb.y) == entity_id);
    let first = hits.next()?;
    hits.next().is_none().then_some((first.x, first.y))
}

/// Ruling EQ item 2: as [`label_cell`], for a placed icon.
pub(crate) fn icon_cell(icons: &[cartalith_assets::manual::ManualIcon], entity_id: i64) -> Option<(f64, f64)> {
    let mut hits = icons.iter().filter(|ic| cartalith_vault::links::icon_entity_id(ic.x, ic.y) == entity_id);
    let first = hits.next()?;
    hits.next().is_none().then_some((first.x, first.y))
}

/// `(x, y)` for the landmark `entity_id` addresses, in grid cells.
///
/// A one-line wrapper, and it exists for one reason: [`WorldGen::entity_cell`]
/// cannot be unit-tested — `WorldGen` is a cdylib `GodotClass` — so the `(x,
/// y)` read inside it is unreachable by any test in this workspace.
/// **Measured**: with the tuple written inline there, mutating it to `(lm.y,
/// lm.x)` **SURVIVED** the suite; against this function it is killed by
/// `a_landmarks_cell_is_x_then_y`. The bounds check stays in `entity_cell`,
/// which is the only caller that knows the grid.
pub(crate) fn landmark_cell(
    landmarks: &[cartalith_civ::landmark::Landmark],
    entity_id: i64,
) -> Option<(i64, i64)> {
    let lm = landmark_with_entity_id(landmarks, entity_id)?;
    Some((lm.x as i64, lm.y as i64))
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

/// `"Yes"` / `"No"` for a boolean readout; never an empty string.
fn yes_no(b: bool) -> String {
    if b { "Yes".to_string() } else { "No".to_string() }
}

/// `s` with its first character upper-cased (Unicode-aware) and the rest
/// untouched; empty in, empty out.
fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Unit tests for the helpers in this file that need no live Godot runtime;
/// everything behind a `#[func]` is reached by probe scenes instead.
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: `thousands` groups digits in threes from the right, including the
    /// no-separator edge (100), the first separator (1,000), zero and negatives.
    #[test]
    fn thousands_groups_from_the_right() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(8420), "8,420");
        assert_eq!(thousands(100), "100");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
        assert_eq!(thousands(-8420), "-8,420");
    }

    /// Protects: `status_rank` ordering -- `Unbound` first and `Connected` last, so
    /// a link set's badge shows the loudest state and `Connected` only when nothing
    /// else applies.
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
    ///
    /// **This covers the input path only, and the distinction is load-bearing
    /// as of 2026-09-06.** `kind_of` reads a kind a *person* just typed or a
    /// panel just passed, and refusing it produces an `error` they see. A kind
    /// read from a *file* is the opposite case and now has the opposite
    /// answer: `SAVEFILE_COMPAT.md` §13.3.5 keeps it, and
    /// `cartalith_vault::LinkKind` is where. Before that change the two shared
    /// one answer and the file half was wrong —
    /// `a_link_kind_this_build_does_not_know_is_kept_and_reported` below is
    /// its counterpart, and
    /// `cartalith_vault::links::tests::a_newer_writers_entity_kind_costs_neither_the_link_nor_the_store`
    /// is the storage proof.
    ///
    /// Protects: that an unrecognised or wrongly-cased entity kind string parses to
    /// `None` rather than a default (and that known kinds still parse).
    #[test]
    fn an_unknown_entity_kind_is_none_not_a_default() {
        assert_eq!(EntityKind::parse("settlement"), Some(EntityKind::Settlement));
        assert_eq!(EntityKind::parse("continent"), Some(EntityKind::Continent));
        assert_eq!(EntityKind::parse("poi"), None, "POI is not a ported concept");
        assert_eq!(EntityKind::parse("Settlement"), None);
        assert_eq!(EntityKind::parse(""), None);
    }

    /// The bridge half of `SAVEFILE_COMPAT.md` §13.3.5: the sentence a person
    /// actually reads when a project names a kind this build does not know.
    ///
    /// `vault_restore_state` cannot run here (it needs a live `WorldGen`), so
    /// this exercises the part that carries the decision —
    /// [`unresolvable_kind_report`] — against a real store built from a real
    /// document, and asserts the two things the sentence must not get wrong:
    /// it must say **nothing was discarded**, because nothing is, and it must
    /// name the kinds rather than only counting them.
    ///
    /// `None` on a store with nothing to report is the other half, and it is
    /// the one that keeps the warning meaningful: a message on every open is
    /// a message nobody reads.
    ///
    /// Protects: `unresolvable_kind_report`'s exact sentence (count, sorted
    /// distinct kinds, "Nothing was discarded"), de-duplication of kinds, and
    /// silence (`None`) for an empty or fully-known store.
    #[test]
    fn a_link_kind_this_build_does_not_know_is_kept_and_reported() {
        let doc = r#"{"version":1,"vaults":[{"id":"v","display_name":"Elaris"}],"links":[
            {"link_id":"a","vault_id":"v","relative_path":"Locations/Nareth.md",
             "entity_kind":"settlement","entity_id":42,"entity_label":"Nareth",
             "selection":{"type":"whole_document"}},
            {"link_id":"b","vault_id":"v","relative_path":"Roads/The Long Way.md",
             "entity_kind":"road","entity_id":9,"entity_label":"The Long Way",
             "selection":{"type":"whole_document"}},
            {"link_id":"c","vault_id":"v","relative_path":"Rivers/The Silverflow.md",
             "entity_kind":"river","entity_id":7,"entity_label":"The Silverflow",
             "selection":{"type":"whole_document"}}]}"#;
        let store = cartalith_vault::LinkStore::from_json(doc).expect("the store still opens");
        assert_eq!(store.links.len(), 3);

        let report = unresolvable_kind_report(&store).expect("two links are unresolvable");
        assert_eq!(
            report,
            "2 of this project's 3 knowledge links name an entity kind this build does not know \
             (river, road); they are kept, listed in the vault window's overview under the name \
             they were saved with, and written back unchanged. Nothing was discarded."
        );

        // The kinds are the *distinct* ones, **sorted** -- the document above
        // lists `road` before `river` on purpose, so a report that echoed the
        // document's own order would read "(road, river)" and fail here.
        // Distinct, too: not one entry per link.
        let both = cartalith_vault::LinkStore::from_json(&doc.replace("\"road\"", "\"river\""))
            .expect("opens");
        let r = unresolvable_kind_report(&both).expect("still two");
        assert!(r.starts_with("2 of this project's 3 "), "{r}");
        assert!(r.contains("(river);"), "one kind, named once: {r}");

        // Nothing to report is silence, not an empty warning.
        assert_eq!(unresolvable_kind_report(&cartalith_vault::LinkStore::default()), None);
        let clean = cartalith_vault::LinkStore::from_json(
            r#"{"version":1,"vaults":[],"links":[
                {"link_id":"a","vault_id":"v","relative_path":"a.md","entity_kind":"landmark",
                 "entity_id":1,"entity_label":"Waterfall","selection":{"type":"whole_document"}}]}"#,
        )
        .expect("opens");
        assert_eq!(unresolvable_kind_report(&clean), None, "every kind known is nothing to say");
    }

    /// One landmark, for the three tests below. Every field but `kind`, `x`
    /// and `y` is deliberately *wrong* in the second run — `id`, `score`,
    /// `importance`, `elevation` and `seed` all move — because those are
    /// exactly the members a resolver must not be reading.
    fn lm(id: u64, kind: &str, x: usize, y: usize) -> cartalith_civ::landmark::Landmark {
        cartalith_civ::landmark::Landmark {
            id,
            kind: kind.to_string(),
            class: cartalith_civ::landmark::LandmarkClass::Regional,
            x,
            y,
            elevation: 100.0 * id as f64,
            score: 0.5,
            importance: 0.5,
            causal: vec!["a".into(), "b".into()],
            seed: id,
        }
    }

    /// **Owner ruling 13's central claim, measured rather than asserted.** A
    /// landmark's note survives a landmark pass that moves every *other*
    /// landmark.
    ///
    /// The second run is deliberately hostile to anything but the key: the
    /// kept landmark's `id` goes 3 -> 1, its `elevation` and `seed` move with
    /// it, two neighbours are gone, three new ones appear, and the list is in
    /// a different order. `key()` — `"peak@40,50"` — is the only thing that
    /// did not move, and the lookup still lands on it.
    ///
    /// Protects: that a landmark is found by its stable key even when its `id`,
    /// order and neighbours all changed, and that a moved landmark is a different
    /// entity (not silently re-bound).
    #[test]
    fn a_landmark_note_survives_a_rerun_that_moves_the_others() {
        let first = [
            lm(1, "waterfall", 10, 12),
            lm(2, "cliff", 20, 22),
            lm(3, "peak", 40, 50),
            lm(4, "mine", 60, 61),
        ];
        let kept = cartalith_vault::links::landmark_entity_id("peak@40,50");
        assert_eq!(landmark_with_entity_id(&first, kept).map(|l| l.id), Some(3));

        // A second pass: a lower cap dropped two, three others were placed,
        // and the survivor's own `id` is now 1.
        let second = [
            lm(1, "peak", 40, 50),
            lm(2, "waterfall", 11, 12),
            lm(3, "gorge", 70, 70),
            lm(4, "spring", 5, 5),
            lm(5, "harbour", 80, 3),
        ];
        let found = landmark_with_entity_id(&second, kept).expect("the kept landmark still resolves");
        assert_eq!(found.key(), "peak@40,50");
        assert_eq!(found.id, 1, "and it is found in spite of its id having moved");
        assert_eq!(found.elevation, 100.0, "the row really is the second run's, not a stale reference");

        // The waterfall moved one cell, so its key moved with it — which is
        // the honest answer: that is a different feature, not the same one
        // relocated, and §13.3.5 wants it reported rather than re-bound
        // behind the user's back.
        let moved = cartalith_vault::links::landmark_entity_id("waterfall@10,12");
        assert!(landmark_with_entity_id(&second, moved).is_none());
        assert!(landmark_with_entity_id(&second, cartalith_vault::links::landmark_entity_id("waterfall@11,12")).is_some());
    }

    /// A note whose landmark is not in the current run is an **absent
    /// entity**, not an error and not a deletion.
    ///
    /// Three ways to be absent, and all three answer the same `None`: the
    /// kind was disarmed, the cap dropped it, or no pass has run at all —
    /// which is the state `WorldGen::generate` and `absorb` leave behind
    /// (`LandmarkStore::invalidate`), so it is the *ordinary* one after a
    /// regenerate rather than an edge case.
    ///
    /// Protects: that an absent landmark (no pass run, disarmed kind, other cell)
    /// resolves to `None` and that hostile ids (`i64::MIN`, 0) do not panic.
    #[test]
    fn a_landmark_that_is_no_longer_placed_is_absent_not_an_error() {
        let gone = cartalith_vault::links::landmark_entity_id("waterfall@120,64");
        // No pass has run.
        assert!(landmark_with_entity_id(&[], gone).is_none());
        // The kind was disarmed; everything else is still there.
        let run = [lm(1, "peak", 120, 64), lm(2, "cliff", 3, 4)];
        assert!(landmark_with_entity_id(&run, gone).is_none(), "same cell, different kind, different entity");
        // And nothing in the resolver can panic on the way: it is one
        // `#[func]` from the gdext boundary.
        assert!(landmark_with_entity_id(&run, i64::MIN).is_none());
        assert!(landmark_with_entity_id(&run, 0).is_none());
    }

    /// `(x, y)`, in that order, and asymmetric fixtures so a swap cannot
    /// pass. The one thing `WorldGen::entity_cell` adds on top — the
    /// grid-bounds check — stays there and is not reachable from here.
    /// Protects: the `(x, y)` order of `landmark_cell` (asymmetric fixtures) and
    /// that an absent landmark has no cell rather than the origin.
    /// Protects Ruling EQ item 2's resolution: `label_cell` and `icon_cell`
    /// find the item by the id of its own cell and answer `(x, y)` in that
    /// order (asymmetric fixtures, fractional positions), answer `None` for an
    /// id nothing holds (never the origin), and **refuse an ambiguous id**
    /// (two labels or two icons in one cell) rather than take the first.
    #[test]
    fn a_label_or_icon_resolves_by_its_cell_and_refuses_a_shared_one() {
        use cartalith_assets::manual::{IconOrigin, ManualIcon, ManualIconFamily};
        use cartalith_civ::labels::MapLabel;
        let labels = [MapLabel::new(12.5, 40.0, "Old Town"), MapLabel::new(3.0, 9.25, "Marsh")];
        assert_eq!(label_cell(&labels, cartalith_vault::links::label_entity_id(12.5, 40.0)), Some((12.5, 40.0)));
        assert_eq!(label_cell(&labels, cartalith_vault::links::label_entity_id(3.0, 9.25)), Some((3.0, 9.25)));
        assert_eq!(label_cell(&labels, cartalith_vault::links::label_entity_id(0.0, 0.0)), None);
        assert_eq!(label_cell(&[], 1), None);
        let twin = [MapLabel::new(1.0, 2.0, "A"), MapLabel::new(1.0, 2.0, "B")];
        assert_eq!(label_cell(&twin, cartalith_vault::links::label_entity_id(1.0, 2.0)), None, "two labels in one cell: refused");
        let icon = |x: f64, y: f64| ManualIcon { x, y, family: ManualIconFamily::Feature, slot: "pine".into(), set: None, scale: 1.0, origin: IconOrigin::Manual };
        let icons = [icon(7.0, 9.25), icon(20.5, 4.0)];
        assert_eq!(icon_cell(&icons, cartalith_vault::links::icon_entity_id(20.5, 4.0)), Some((20.5, 4.0)));
        assert_eq!(icon_cell(&icons, cartalith_vault::links::icon_entity_id(7.0, 9.25)), Some((7.0, 9.25)));
        assert_eq!(icon_cell(&icons, cartalith_vault::links::label_entity_id(7.0, 9.25)), None, "a label id is not an icon id");
        let stacked = [icon(5.0, 5.0), icon(5.0, 5.0)];
        assert_eq!(icon_cell(&stacked, cartalith_vault::links::icon_entity_id(5.0, 5.0)), None, "two icons in one cell: refused");
    }

    #[test]
    fn a_landmarks_cell_is_x_then_y() {
        let run = [lm(1, "peak", 40, 50), lm(2, "cliff", 7, 3)];
        assert_eq!(
            landmark_cell(&run, cartalith_vault::links::landmark_entity_id("peak@40,50")),
            Some((40, 50))
        );
        assert_eq!(
            landmark_cell(&run, cartalith_vault::links::landmark_entity_id("cliff@7,3")),
            Some((7, 3))
        );
        // And an absent landmark has no cell rather than the origin.
        assert_eq!(landmark_cell(&run, cartalith_vault::links::landmark_entity_id("mine@0,0")), None);
        assert_eq!(landmark_cell(&[], 1), None);
    }

    /// An ambiguous id resolves to **nothing**, not to the first match.
    ///
    /// `landmark_entity_id` is 52 bits, so two keys *can* collide; a real run
    /// also cannot contain two rows with one key
    /// (`the_stable_key_is_unique_within_a_run` measures that on a world).
    /// Both reach this branch, and the branch exists because taking the first
    /// match would attach someone's note to the wrong feature silently, while
    /// `None` makes it an unresolved link they can re-bind.
    ///
    /// Protects: the ambiguity rule -- two rows claiming one id resolve to
    /// nothing, while an unambiguous neighbour in the same list still resolves.
    #[test]
    fn an_ambiguous_landmark_id_resolves_to_nothing() {
        let id = cartalith_vault::links::landmark_entity_id("peak@40,50");
        assert!(landmark_with_entity_id(&[lm(1, "peak", 40, 50)], id).is_some());
        let twice = [lm(1, "peak", 40, 50), lm(2, "cliff", 9, 9), lm(3, "peak", 40, 50)];
        assert!(landmark_with_entity_id(&twice, id).is_none(), "two claimants means no claimant");
        // The unambiguous neighbour in the same list is unaffected.
        assert!(
            landmark_with_entity_id(&twice, cartalith_vault::links::landmark_entity_id("cliff@9,9")).is_some()
        );
    }

    /// `GUI_GAP_REGISTER.md` **CV-02**: the kind the vault gained on
    /// 2026-08-25, and the id it is addressed by.
    ///
    /// The assertion that matters is the second one. A culture's id is an
    /// index into a **compile-time** table, so if that table ever changes
    /// length or order every culture link in every user's sidecar silently
    /// re-points at a different culture — the exact failure `entity_label`
    /// exists to make visible and this test exists to make loud first.
    ///
    /// Protects: the culture kind's id scheme -- parse/`as_str`/`entity_key`
    /// round trip and the literal length (7) and order of `CIV_CULTURES`, which
    /// existing links index into, plus the absence of a terrain verdict for the two
    /// identity cultures.
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

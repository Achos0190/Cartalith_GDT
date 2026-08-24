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
//! ## Where the links live, and the honest limitation
//!
//! `vault_state_json()`/`vault_restore_state()` hand the whole link store to
//! GDScript as text; the shell persists it to `user://markdown_vault.json`.
//!
//! That is **profile-scoped, not project-scoped**, and the reason is worth
//! stating rather than discovering: `cartalith-io`'s save format is the
//! reference HTML app's own `.zip` (`SAVEFILE_COMPAT.md`) and carries no civ
//! data at all — `WorldGen::load_save` produces a world whose
//! `get_settlements()` is empty. A link stored *inside* a save would come back
//! pointing at settlements that no longer exist. Until the save format carries
//! the civ layer, there is no project for a project-scoped link store to
//! belong to. `MARKDOWN_VAULT_SCOPE.md` carries this as milestone 3.

use crate::WorldGen;
use cartalith_vault::{export, links::entity_key, BlockAction, EntityKind, FieldFill, FieldOutcome, LinkStatus, Selection};
use godot::prelude::*;
use std::collections::BTreeMap;

/// `{"ok": false, "error": …}` — the single failure shape every method here
/// returns, so GDScript has one thing to check.
fn err(message: impl std::fmt::Display) -> VarDictionary {
    vdict! { "ok" => false, "error" => message.to_string() }
}

fn ok() -> VarDictionary {
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

    /// `{bound, root, display_name, vault_id, link_count}`. `root` is the
    /// device-local path and is for showing the user where they pointed
    /// Cartalith, never for storing in project data (§5).
    #[func]
    fn vault_info(&self) -> VarDictionary {
        let v = self.vault.store.vaults.first();
        vdict! {
            "bound" => self.vault.is_bound(),
            "root" => self.vault.vault().map(|v| v.root().display().to_string()).unwrap_or_default(),
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

    // -- links (§11, §12, §13) ---------------------------------------------

    /// Attaches an entity to a document, or to one heading section of it.
    ///
    /// `kind` is `"settlement"`/`"province"`/`"continent"`; `entity_id` is
    /// that kind's own id (a settlement's `tid`, a province's `id`, a
    /// continent's rank — see `get_continents()` on what that means).
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
        }
        out
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
}

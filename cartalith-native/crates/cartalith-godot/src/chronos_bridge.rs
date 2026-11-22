//! The chronos timeline's bridge (`CHRONOS_TIMELINE_DESIGN.md`, Ruling GD / GE, CT-2 to CT-8).
//!
//! Thin, like every bridge here: the rules are `cartalith_vault::timelines` (detection, decisions,
//! imported events) and `cartalith_civ::timeline_items` (the read model, the filters, the export),
//! each with its own tests. This file only fills the read model's inputs from the live stores and
//! turns the results into `VarDictionary`s.
//!
//! **Read-only over notes.** Nothing here calls a vault write. `timeline_decide` reads the note it
//! decides about (to hash the block and, for an import, to copy its events) and writes only the
//! link store's decision row and the imported-events list. The one note write in the whole feature
//! stays `vault_add_chronos_event`.
//!
//! **Never touches the year cursor.** The shell moves time with `tl_set_year` as it always has; no
//! function here writes `CivData::year`.

use crate::vault_bridge::{err, ok};
use crate::WorldGen;
use cartalith_civ::timeline_items::{self as ti, ItemSource, ItemSources};
use cartalith_vault::timelines::{self as tl, Decision, Form, Standing, TimelineDecision};
use cartalith_vault::EntityKind;
use godot::prelude::*;

/// How many of a candidate's events the scan reports for the prompt's preview axis (a labelled judgement:
/// a preview is a shape, not a listing).
const PREVIEW_EVENTS_MAX: usize = 400;

fn md_set(d: &mut VarDictionary, prefix: &str, md: Option<cartalith_vault::chronos::MonthDay>) {
    if let Some(m) = md {
        d.set(format!("{prefix}_month").as_str(), m.month as i64);
        if let Some(day) = m.day {
            d.set(format!("{prefix}_day").as_str(), day as i64);
        }
    }
}

/// `YYYY-MM-DD` (UTC) for `secs` since the Unix epoch (Hinnant's civil-from-days).
fn civil_date(secs: i64) -> String {
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}", yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn standing_word(s: Standing) -> (&'static str, &'static str) {
    let w = |d: Decision| match d {
        Decision::Keep => "keep",
        Decision::Ignore => "ignore",
        Decision::Link => "link",
        Decision::Import => "import",
    };
    match s {
        Standing::Undecided => ("undecided", ""),
        Standing::Decided(d) => ("decided", w(d)),
        Standing::Changed(d) => ("changed", w(d)),
    }
}

fn form_of(s: &str) -> Option<Form> {
    [Form::Chronos, Form::Frontmatter, Form::YearList, Form::Table, Form::HtmlSpans].into_iter().find(|f| f.as_str() == s)
}

fn decision_of(s: &str) -> Option<Decision> {
    match s {
        "keep" => Some(Decision::Keep),
        "ignore" => Some(Decision::Ignore),
        "link" => Some(Decision::Link),
        "import" => Some(Decision::Import),
        _ => None,
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// The undo token of one decision: the rows an import added and the decision row it replaced.
#[derive(serde::Serialize, serde::Deserialize)]
struct UndoToken {
    ids: Vec<u64>,
    prev: Option<TimelineDecision>,
    key: (String, String, Form, usize),
}

#[godot_api(secondary)]
impl WorldGen {
    /// The id the bound vault is filed under in `vault.json` (the first registered one, as
    /// `vault_info()` reports), or `None` with no vault.
    fn ct_vault_id(&self) -> Option<String> {
        self.vault.store.vaults.first().map(|v| v.id.clone())
    }

    /// Scans a slice of the bound vault for timelines (CT-3), **read-budgeted**: at most
    /// `max_reads` notes (`<= 0` means 200) starting at note number `offset`.
    ///
    /// `{ok, error, next, scanned, candidates: [{rel, form, form_label, ordinal, count, skipped,
    /// year_min?, year_max?, heading?, hash, preview: [String], linked, private?, standing,
    /// previous}], badge}`. `next` is the offset for the following call, `-1` when the whole list
    /// is covered. `standing` is `undecided`, `decided` or `changed`; `previous` is the saved
    /// decision word (empty when none). `badge` counts the candidates in this slice that still
    /// need attention (the passive "N timelines found" badge, which never opens a prompt).
    /// `private` is the fixed-vocabulary label of `private::PrivateMarker`, never note text. A note
    /// not read in this call is not in `candidates` and **not decided**; the caller keeps its
    /// offset rather than concluding "no timeline".
    #[func]
    fn timelines_scan(&self, offset: i64, max_reads: i64) -> VarDictionary {
        let Some(vid) = self.ct_vault_id() else { return err("no vault is connected") };
        let budget = if max_reads > 0 { max_reads as usize } else { 200 };
        let (scan, next) = match self.vault.scan_timelines(offset.max(0) as usize, budget) {
            Ok(x) => x,
            Err(e) => return err(e),
        };
        let mut badge = 0;
        let mut rows: Array<VarDictionary> = Array::new();
        for n in &scan.found {
            let linked = self.vault.store.links.iter().any(|l| l.relative_path == n.path);
            for c in &n.candidates {
                let st = tl::standing(&self.vault.store.timelines, &vid, &n.path, c);
                if st.needs_attention() {
                    badge += 1;
                }
                let (word, prev) = standing_word(st);
                let preview: PackedStringArray = c.preview().iter().map(|s| GString::from(s.as_str())).collect();
                let mut d = vdict! {
                    "rel" => n.path.as_str(),
                    "form" => c.form.as_str(),
                    "form_label" => c.form.label(),
                    "ordinal" => c.ordinal as i64,
                    "count" => c.events.len() as i64,
                    "skipped" => c.skipped.len() as i64,
                    "hash" => c.block_hash.as_str(),
                    "linked" => linked,
                    "standing" => word,
                    "previous" => prev,
                };
                d.set("preview", &preview);
                if let Some((lo, hi)) = c.year_range() {
                    d.set("year_min", lo);
                    d.set("year_max", hi);
                }
                if let Some(h) = &c.heading {
                    d.set("heading", h.as_str());
                }
                if let Some(m) = &n.private {
                    d.set("private", m.label().as_str());
                }
                if let Some(f) = c.founded_year() {
                    d.set("founded", f);
                }
                // Ruling GT: the events' years, for the prompt's preview on the shared axis. Capped at
                // `PREVIEW_EVENTS_MAX` so a 10 000-line table cannot bloat the scan page; `count` stays the true total.
                let ev: Array<VarDictionary> = c
                    .events
                    .iter()
                    .take(PREVIEW_EVENTS_MAX)
                    .map(|e| {
                        let mut v = vdict! { "start" => e.start, "title" => e.name.as_str() };
                        if let Some(end) = e.end {
                            v.set("end", end);
                        }
                        v
                    })
                    .collect();
                d.set("events", &ev);
                rows.push(&d);
            }
        }
        let mut out = ok();
        out.set("candidates", &rows);
        out.set("scanned", scan.decided.len() as i64);
        out.set("next", next.map_or(-1, |n| n as i64));
        out.set("badge", badge as i64);
        out
    }

    /// Records the user's choice for one timeline and, for `import`, copies its events onto the map
    /// (CT-4, CT-5). `decision` is `keep`, `ignore`, `link` or `import`; `entity_kind`/`entity_id`
    /// bind a `link` or `import` (a kind `EntityKind::parse` knows); `epoch_offset` is added to the
    /// note's years on import (default 0). **The note is read, never written.**
    ///
    /// `{ok, error, added: PackedInt64Array, duplicates, undo}`. `undo` is an opaque token for
    /// `timeline_decide_undo`, so one click is one undo step (the import's rows and the decision
    /// row together). A private note's import is saved flagged `private` and the call also returns
    /// `private_warning` (the one-line warning the prompt shows). The block must still be there: a
    /// note that changed so the `form`/`ordinal` no longer exists is an `error`.
    #[func]
    fn timeline_decide(&mut self, rel: GString, form: GString, ordinal: i64, decision: GString, entity_kind: GString, entity_id: i64, epoch_offset: i64) -> VarDictionary {
        let Some(vid) = self.ct_vault_id() else { return err("no vault is connected") };
        let (Some(form), Some(dec)) = (form_of(&form.to_string()), decision_of(&decision.to_string())) else {
            return err("unknown form or decision");
        };
        let rel = rel.to_string();
        let Ok(text) = self.vault.read(&rel) else { return err(format!("{rel} could not be read")) };
        let found = tl::detect(&text);
        let Some(c) = found.iter().find(|c| c.form == form && c.ordinal == ordinal.max(0) as usize) else {
            return err("that timeline is no longer in the note");
        };
        let kind = entity_kind.to_string();
        let bound = matches!(dec, Decision::Link | Decision::Import);
        if bound && EntityKind::parse(&kind).is_none() {
            return err(format!("unknown entity kind {kind}"));
        }
        let key = (vid.clone(), rel.clone(), form, c.ordinal);
        let prev = self.vault.store.timelines.iter().find(|d| d.vault_id == key.0 && d.relative_path == key.1 && d.form == key.2 && d.ordinal == key.3).cloned();
        let private = cartalith_vault::private::private_marker(&rel, &text).is_some();
        let mut ids: Vec<u64> = Vec::new();
        let mut duplicates = 0;
        if dec == Decision::Import {
            let r = self.vault.events.import(c, &vid, &rel, &kind, entity_id, epoch_offset, private);
            ids = r.added;
            duplicates = r.duplicates;
        }
        tl::record(
            &mut self.vault.store.timelines,
            TimelineDecision {
                vault_id: vid,
                relative_path: rel,
                form,
                ordinal: c.ordinal,
                block_hash: c.block_hash.clone(),
                decision: dec,
                entity_kind: bound.then(|| kind.clone()),
                entity_id: bound.then_some(entity_id),
                epoch_offset: (dec == Decision::Import).then_some(epoch_offset),
                decided: now_secs(),
            },
        );
        let token = serde_json::to_string(&UndoToken { ids: ids.clone(), prev, key }).unwrap_or_default();
        let mut out = ok();
        out.set("added", &ids.iter().map(|i| *i as i64).collect::<PackedInt64Array>());
        out.set("duplicates", duplicates as i64);
        out.set("undo", token.as_str());
        if private && bound {
            out.set("private_warning", "this note is private; its events will be saved in the project file");
        }
        out
    }

    /// Undoes one `timeline_decide`: removes the rows it imported and restores the decision row it
    /// replaced (or removes the row it created). `{ok, error}`.
    #[func]
    fn timeline_decide_undo(&mut self, token: GString) -> VarDictionary {
        let Ok(t) = serde_json::from_str::<UndoToken>(&token.to_string()) else { return err("not an undo token") };
        self.vault.events.remove_ids(&t.ids);
        let tls = &mut self.vault.store.timelines;
        tls.retain(|d| !(d.vault_id == t.key.0 && d.relative_path == t.key.1 && d.form == t.key.2 && d.ordinal == t.key.3));
        if let Some(p) = t.prev {
            tls.push(p);
        }
        ok()
    }

    /// The consolidated timeline (CT-1, CT-2): every dated item the map knows, filtered.
    ///
    /// `filter` keys, all optional (absent = no restriction on that axis; groups AND, values OR):
    /// `kinds` (`PackedStringArray` of `Founding`, `Snapshot`, `Ownership`, `Conflict`, `Journey`,
    /// `Note`, `Imported`), `entities` (`PackedStringArray` of `settlement:42`-style keys), `factions`
    /// (`PackedInt32Array`). **The ownership lane is lazy**: its spans are computed only when
    /// `kinds` names `Ownership`, because they are derived from every snapshot per settlement.
    ///
    /// `{ok, items: [{key, source, start_year, start_month?, start_day?, end_year?, end_month?,
    /// end_day?, open_end, title, entity_kind?, entity_id?, factions, precision, private,
    /// rel?}], hidden, total, year_min?, year_max?}` (the extent is of the rows the filter keeps, not of every item). **Optional keys are absent, never 0.**
    /// A founding for a settlement with no founding year is absent (never year 0). A journey is a range
    /// when its saved plan gives a timeline (`arrival_offset_days`), else a point.
    /// Items from a private note carry `private: true`; the assistant's tools and every export must
    /// skip them (`timeline_export` does).
    #[func]
    fn timeline_items(&mut self, filter: VarDictionary) -> VarDictionary {
        let (items, f) = self.ct_items(&filter);
        let shown = ti::apply(&items, &f);
        let mut rows: Array<VarDictionary> = Array::new();
        for i in &shown.visible {
            let mut d = vdict! {
                "key" => i.key.as_str(),
                "source" => i.source.label(),
                "start_year" => i.start.0,
                "open_end" => i.open_end,
                "title" => i.title.as_str(),
                "precision" => match i.precision { ti::Precision::Year => "year", ti::Precision::Date => "date", ti::Precision::Range => "range" },
                "private" => i.private,
                "also_linked" => i.also_linked,
                "lane" => ti::lane_of(i),
            };
            // Ruling GT: the Chronos kind word (`event`/`period`/`point`/`marker`) and the `#colour` tag, absent
            // (never "") for a row Cartalith derived itself
            if let Some(k) = i.ev_kind {
                d.set("ev_kind", k.as_str());
            }
            if let Some(c) = &i.color {
                d.set("color", c.as_str());
            }
            md_set(&mut d, "start", i.start.1);
            if let Some((y, md)) = i.end {
                d.set("end_year", y);
                md_set(&mut d, "end", md);
            }
            if let Some((k, id)) = i.entity {
                d.set("entity_kind", k.as_str());
                d.set("entity_id", id);
            }
            d.set("factions", &i.factions.iter().copied().collect::<PackedInt32Array>());
            if let Some(o) = &i.origin {
                d.set("rel", o.relative_path.as_str());
            }
            rows.push(&d);
        }
        let mut out = ok();
        out.set("items", &rows);
        out.set("hidden", shown.hidden as i64);
        out.set("total", items.len() as i64);
        // the extent of what the filter KEEPS, not of every item: the strip asks for the Note and Imported lanes
        // alone (CT view issue 1), and a founding at -635 must not widen that answer
        if let Some((lo, hi)) = ti::year_extent(shown.visible.iter().copied()) {
            out.set("year_min", lo);
            out.set("year_max", hi);
        }
        out
    }

    /// Exports the filtered view as a pasteable Chronos block (CT-8): `{ok, text, written,
    /// skipped: PackedStringArray}`. **Private items are never written.** The text carries a header
    /// comment saying it is generated and not read back; it is not stored anywhere.
    #[func]
    fn timeline_export(&mut self, filter: VarDictionary) -> VarDictionary {
        let (items, f) = self.ct_items(&filter);
        let shown = ti::apply(&items, &f);
        let x = ti::export_chronos(&shown.visible);
        let mut out = ok();
        out.set("text", x.text.as_str());
        out.set("written", x.written as i64);
        out.set("skipped", &x.skipped.iter().map(|s| GString::from(s.as_str())).collect::<PackedStringArray>());
        out
    }

    /// Ruling GT, L3: writes the consolidated timeline as `_cartalith/Consolidated timeline.md` in the bound
    /// vault and returns its text, **only when asked** (the strip's "Open as document"), regenerated each time.
    /// A file already at that path **without the `cartalith: generated-timeline` frontmatter marker is never
    /// replaced**: `{ok: false, refused: true, error}` and the file is untouched. Private items are left out
    /// (`ti::consolidated_document`). `{ok, error, refused?, rel, text, written, skipped}`.
    #[func]
    fn timeline_open_document(&mut self, world: GString) -> VarDictionary {
        const REL: &str = "_cartalith/Consolidated timeline.md";
        let (items, _f) = self.ct_items(&VarDictionary::new());
        let refs: Vec<&ti::TimelineItem> = items.iter().collect();
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        let doc = ti::consolidated_document(&refs, &world.to_string(), &civil_date(secs));
        match self.vault.write_generated(REL, &doc.text, ti::is_generated_document) {
            Ok(true) => {
                let mut out = ok();
                out.set("rel", REL);
                out.set("text", doc.text.as_str());
                out.set("written", doc.written as i64);
                out.set("skipped", &doc.skipped.iter().map(|s| GString::from(s.as_str())).collect::<PackedStringArray>());
                out
            }
            Ok(false) => {
                let mut out = err(format!("{REL} already exists and is not a Cartalith generated timeline, so it was left as it is. Rename or move it to let Cartalith write this file."));
                out.set("refused", true);
                out
            }
            Err(e) => err(format!("the consolidated timeline could not be written: {e}")),
        }
    }

    /// What a re-sync of the note's imported timeline would do (CT-7): `{ok, added, removed,
    /// changed}` counts, plus `added_rows` (`[{title, year}]`). Shown, never applied silently.
    /// `entity_kind`/`entity_id` name the item the rows were imported onto.
    #[func]
    fn timeline_resync_plan(&self, rel: GString, form: GString, ordinal: i64, entity_kind: GString, entity_id: i64) -> VarDictionary {
        let Some(vid) = self.ct_vault_id() else { return err("no vault is connected") };
        let Some(c) = self.ct_candidate(&rel.to_string(), &form.to_string(), ordinal) else { return err("that timeline is no longer in the note") };
        let plan = self.vault.events.resync_plan(&c, &vid, &rel.to_string(), &entity_kind.to_string(), entity_id);
        let rows: Array<VarDictionary> = plan.added.iter().map(|e| vdict! { "title" => e.name.as_str(), "year" => e.start }).collect();
        let mut out = ok();
        out.set("added", plan.added.len() as i64);
        out.set("removed", plan.removed.len() as i64);
        out.set("changed", plan.changed.len() as i64);
        out.set("added_rows", &rows);
        out
    }

    /// Applies the chosen groups of the re-sync plan (CT-7): `take_added`, `take_changed`,
    /// `take_removed`. A removed line's row is **kept** and marked `source_removed`. Returns
    /// `{ok, error, undo}` where `undo` is a token for `timeline_resync_undo` (the whole imported
    /// list as it was): one click, one undo step.
    #[func]
    fn timeline_resync_apply(&mut self, rel: GString, form: GString, ordinal: i64, entity_kind: GString, entity_id: i64, take_added: bool, take_changed: bool, take_removed: bool) -> VarDictionary {
        let Some(vid) = self.ct_vault_id() else { return err("no vault is connected") };
        let (rel, kind) = (rel.to_string(), entity_kind.to_string());
        let Some(c) = self.ct_candidate(&rel, &form.to_string(), ordinal) else { return err("that timeline is no longer in the note") };
        let plan = self.vault.events.resync_plan(&c, &vid, &rel, &kind, entity_id);
        let before = self.vault.events.to_json();
        let private = self.vault.read(&rel).map(|t| cartalith_vault::private::private_marker(&rel, &t).is_some()).unwrap_or(true);
        self.vault.events.apply_resync(&plan, take_added, take_changed, take_removed, &vid, &rel, &kind, entity_id, private);
        let mut out = ok();
        out.set("undo", before.as_str());
        out
    }

    /// Restores the imported list from `timeline_resync_apply`'s token. `{ok, error}`.
    #[func]
    fn timeline_resync_undo(&mut self, token: GString) -> VarDictionary {
        match tl::EventsDoc::from_json(&token.to_string()) {
            Ok((doc, 0)) => {
                self.vault.events = doc;
                ok()
            }
            _ => err("not a resync undo token"),
        }
    }

    /// How a note's founding events compare with settlement `tid`'s own founding year (CT-7,
    /// design §4.5): `[{title, note_year, founded?, verdict}]`, `verdict` one of `agrees`,
    /// `conflict`, `not_applicable`. Both years stay visible on a conflict; nothing is resolved.
    #[func]
    fn timeline_founding_check(&self, rel: GString, form: GString, ordinal: i64, tid: i64) -> Array<VarDictionary> {
        let founded = self.civ.as_ref().and_then(|civ| civ.place_extras.0.get(&(tid.max(0) as u64)).and_then(|e| e.founded));
        let Some(c) = self.ct_candidate(&rel.to_string(), &form.to_string(), ordinal) else { return Array::new() };
        c.events
            .iter()
            .map(|e| {
                let (verdict, note) = match tl::founding_check(founded, e) {
                    tl::FoundingCheck::NotApplicable => ("not_applicable", e.start),
                    tl::FoundingCheck::Agrees => ("agrees", e.start),
                    tl::FoundingCheck::Conflict { note, .. } => ("conflict", note),
                };
                let mut d = vdict! { "title" => e.name.as_str(), "note_year" => note, "verdict" => verdict };
                if let Some(f) = founded {
                    d.set("founded", f);
                }
                d
            })
            .collect()
    }

    /// The imported events' row count and private count, for the Items list header: `{ok, imported,
    /// private}`.
    #[func]
    fn timeline_imported_counts(&self) -> VarDictionary {
        let e = &self.vault.events.events;
        let mut out = ok();
        out.set("imported", e.len() as i64);
        out.set("private", e.iter().filter(|r| r.private).count() as i64);
        out
    }

    /// Re-reads one candidate from the note by `(form, ordinal)`.
    fn ct_candidate(&self, rel: &str, form: &str, ordinal: i64) -> Option<tl::Candidate> {
        let form = form_of(form)?;
        let text = self.vault.read(rel).ok()?;
        tl::detect(&text).into_iter().find(|c| c.form == form && c.ordinal == ordinal.max(0) as usize)
    }

    /// Builds the read model's inputs from the live stores and parses `filter`.
    fn ct_items(&mut self, filter: &VarDictionary) -> (Vec<ti::TimelineItem>, ti::Filter) {
        let mut f = ti::Filter::default();
        if let Some(v) = filter.get("kinds").and_then(|v| v.try_to::<PackedStringArray>().ok()) {
            f.kinds = v.as_slice().iter().filter_map(|s| ItemSource::parse(&s.to_string())).collect();
        }
        if let Some(v) = filter.get("entities").and_then(|v| v.try_to::<PackedStringArray>().ok()) {
            f.entities = v
                .as_slice()
                .iter()
                .filter_map(|s| {
                    let s = s.to_string();
                    let (k, id) = s.split_once(':')?;
                    Some((EntityKind::parse(k)?, id.parse::<i64>().ok()?))
                })
                .collect();
        }
        if let Some(v) = filter.get("factions").and_then(|v| v.try_to::<PackedInt32Array>().ok()) {
            f.factions = v.as_slice().to_vec();
        }

        let mut src = ItemSources::default();
        if let Some(civ) = self.civ.as_ref() {
            for s in &civ.settlements {
                src.founding.push(ti::FoundingInput {
                    id: s.tid as i64,
                    name: s.name.clone(),
                    founded: civ.place_extras.0.get(&s.tid).and_then(|e| e.founded),
                    faction: s.placement.faction,
                });
            }
            src.snapshot_years = civ.timeline.iter().map(|t| t.year).collect();
            if f.kinds.contains(&ItemSource::Ownership) {
                // derived from every snapshot per settlement, so only the settlements the entity
                // filter can keep are computed (an empty entity filter keeps all)
                let wanted = |tid: i64| f.entities.is_empty() || f.entities.contains(&(EntityKind::Settlement, tid));
                for s in civ.settlements.iter().filter(|s| wanted(s.tid as i64)) {
                    src.ownership.push(ti::OwnershipInput { tid: s.tid as i64, name: s.name.clone(), spans: civ.civ_settlement_ownership_periods(s.tid) });
                }
            }
        }
        src.conflicts = self.conflicts.conflicts.clone();
        // A journey's length is the planner's `JourneyTimeline::arrival_offset_days` (whole days, the
        // same figure `journey_positions` reports as `arrival`), planned only when the Journey lane can
        // show (no kind filter, or Journey named); a journey whose plan gives no timeline stays a point.
        let mut lengths: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
        if f.kinds.is_empty() || f.kinds.contains(&ItemSource::Journey) {
            for (ji, _missing, planned) in self.plan_saved_journeys(&|_| true) {
                if let Some(Ok(t)) = planned {
                    lengths.insert(ji, t.arrival_offset_days().clamp(0, i64::from(u32::MAX)) as u32);
                }
            }
        }
        if let Some(infra) = self.infra.as_ref() {
            for (ji, j) in infra.journeys.iter().enumerate() {
                src.journeys.push(ti::JourneyInput { id: j.id, name: j.name.clone(), start_year: j.start_year, start_day: j.start_day, calendar_days: lengths.get(&ji).copied() });
            }
        }
        // Note events: live from disk for every linked entity, one input per (note, entity).
        let mut seen: Vec<(EntityKind, i64)> = Vec::new();
        for l in &self.vault.store.links {
            let Some(k) = l.entity_kind.known() else { continue };
            if seen.contains(&(k, l.entity_id)) {
                continue;
            }
            seen.push((k, l.entity_id));
            let factions: Vec<i32> = match k {
                EntityKind::Faction => vec![l.entity_id as i32],
                EntityKind::Settlement => self
                    .civ
                    .as_ref()
                    .and_then(|c| c.settlements.iter().find(|s| s.tid as i64 == l.entity_id).map(|s| vec![s.placement.faction]))
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            let vid = self.vault.store.vaults.first().map(|v| v.id.clone()).unwrap_or_default();
            for (rel, _live, chronos) in self.vault.entity_chronos(k, l.entity_id) {
                // an unreadable note is "not decided", never "public": it reads as private here
                let private = self.vault.read(&rel).map(|t| cartalith_vault::private::private_marker(&rel, &t).is_some()).unwrap_or(true);
                src.note_events.push(ti::NoteEventsInput {
                    link_id: l.link_id.clone(),
                    vault_id: vid.clone(),
                    relative_path: rel,
                    entity: Some((k, l.entity_id)),
                    factions: factions.clone(),
                    private,
                    events: chronos.events,
                });
            }
        }
        src.imported = self.vault.events.events.clone();
        (ti::build(&src), f)
    }
}

#[cfg(test)]
mod tests {
    /// Protects the L3 document's `generated:` date: literal known days, incl. a leap day and the epoch.
    #[test]
    fn civil_date_is_the_utc_calendar_day() {
        assert_eq!(super::civil_date(0), "1970-01-01");
        assert_eq!(super::civil_date(951_782_400), "2000-02-29");
        assert_eq!(super::civil_date(1_795_305_600 + 3600), "2026-11-22");
    }
}

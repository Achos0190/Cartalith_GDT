//! `WorldGen`'s half of the annotation undo (`annotation_undo.rs` holds the
//! pure logic): reading and writing the three hand-edited stores, recording an
//! edit as a ledger row, undoing and redoing it, and ordering its redo offer
//! against an undone way drag.
//!
//! A second `impl WorldGen` file like `context_pick_bridge.rs`; every method
//! here is crate-internal except the three `#[func]`s at the bottom.
//!
//! ## How an edit is recorded
//!
//! [`WorldGen::anno_scope`] wraps a mutator: it clones the store, runs the
//! mutator, and records one ledger row **only if the store changed**. A refused
//! call (a bad index, an off-grid move) therefore records nothing, and no call
//! site has to guess whether its own edit took.

use std::sync::Arc;

use crate::annotation_undo::{AnnoEntry, AnnoRedo, AnnoSnap, AnnoStep, AnnoStore, Repoint};
use crate::{undo, WorldGen};
use godot::prelude::*;

/// What a redo would re-apply next. The height tail is the third kind
/// (`lib.rs::redo_last` serves it itself), offered after both of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RedoKind {
    /// An undone way drag (`context_pick_bridge::DragRedo`).
    Drag,
    /// An undone label, icon or landmark-name edit.
    Annotation,
    /// An undone "Make this sphere a faction" (`city_state_bridge::SphereRedo`).
    Sphere,
}

/// One ready redo offer that is not the height tail.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RedoOffer {
    /// Which stack it is on.
    pub kind: RedoKind,
    /// `undo::next_step_id` of its undo: what `redo_offer_ids` reports.
    pub id: u64,
    /// The history label.
    pub label: String,
}

/// Orders the ready drag and annotation offers: the one undone **later** (the
/// larger step id) is redone first, which is last-in first-out across the two
/// stacks. Pure so the order is unit-tested without a live engine.
pub(crate) fn order_offers(drag: Option<(u64, String)>, anno: Option<(u64, String)>, sphere: Option<(u64, String)>) -> Vec<RedoOffer> {
    let mut v: Vec<RedoOffer> = Vec::new();
    if let Some((id, label)) = drag {
        v.push(RedoOffer { kind: RedoKind::Drag, id, label });
    }
    if let Some((id, label)) = anno {
        v.push(RedoOffer { kind: RedoKind::Annotation, id, label });
    }
    if let Some((id, label)) = sphere {
        v.push(RedoOffer { kind: RedoKind::Sphere, id, label });
    }
    v.sort_by(|a, b| b.id.cmp(&a.id));
    v
}

/// The history row's words: `Move label “Ash”`, or `Move unnamed label` when the
/// name is blank -- never an empty pair of quotes standing for a name it does not have.
pub(crate) fn row_label(verb: &str, noun: &str, name: &str) -> String {
    if name.trim().is_empty() {
        format!("{verb} unnamed {noun}")
    } else {
        format!("{verb} {noun} “{name}”")
    }
}

impl WorldGen {
    /// The text of hand-placed label `index`; `None` for an index the store does not hold.
    pub(crate) fn anno_label_name(&self, index: i64) -> Option<String> {
        let i = usize::try_from(index).ok()?;
        self.labels.as_ref()?.labels.get(i).map(|l| l.name.clone())
    }

    /// The slot name of placed icon `index` (its only name); `None` for an index the store does not hold.
    pub(crate) fn anno_icon_slot(&self, index: i64) -> Option<String> {
        let i = usize::try_from(index).ok()?;
        self.icons.as_ref()?.icons.get(i).map(|ic| ic.slot.clone())
    }

    /// The current content of one store, cloned; `None` when the store does not
    /// exist (no world yet, or no label bridge).
    pub(crate) fn anno_snap(&self, store: AnnoStore) -> Option<AnnoSnap> {
        match store {
            AnnoStore::Labels => self.labels.as_ref().map(|l| AnnoSnap::Labels(l.labels.clone(), l.overrides.clone())),
            AnnoStore::Icons => self.icons.as_ref().map(|i| AnnoSnap::Icons(i.icons.clone())),
            AnnoStore::Names => Some(AnnoSnap::Names(self.landmark_user_names.clone())),
            AnnoStore::Pois => Some(AnnoSnap::Pois(self.pois.clone())),
            AnnoStore::Kinds => Some(AnnoSnap::Kinds(self.landmark_kind_overrides.clone())),
        }
    }

    /// Whether the live store equals `s`, without cloning it. What every
    /// liveness and redo-readiness check asks.
    pub(crate) fn anno_matches(&self, s: &AnnoSnap) -> bool {
        match s {
            AnnoSnap::Labels(v, o) => self.labels.as_ref().is_some_and(|l| l.labels == *v && l.overrides == *o),
            AnnoSnap::Icons(v) => self.icons.as_ref().is_some_and(|i| i.icons == *v),
            AnnoSnap::Names(m) => self.landmark_user_names == *m,
            AnnoSnap::Pois(v) => self.pois == *v,
            AnnoSnap::Kinds(m) => self.landmark_kind_overrides == *m,
        }
    }

    /// Writes a snapshot back into its store and drops the engine-side
    /// selection of that store (a restored list may have shifted every index,
    /// and a selection that named a row must never name a neighbour). Records
    /// what changed for the shell ([`AnnoStep::shifted`]: the row count moved).
    fn anno_apply(&mut self, s: &AnnoSnap) {
        let was = self.anno_snap(s.store()).map_or(0, |c| c.len());
        match s {
            AnnoSnap::Labels(v, o) => {
                if let Some(l) = self.labels.as_mut() {
                    l.labels = v.clone();
                    l.overrides = o.clone();
                    l.deselect();
                }
            }
            AnnoSnap::Icons(v) => {
                if let Some(i) = self.icons.as_mut() {
                    i.icons = v.clone();
                    i.deselect();
                }
                // The landmark result counts hand-placed icons as obstacles; an
                // icon change is as stale-making as a placement (`icon_place`).
                self.landmark_store.mark_icon_committed();
            }
            AnnoSnap::Names(m) => self.landmark_user_names = m.clone(),
            AnnoSnap::Pois(v) => self.pois = v.clone(),
            AnnoSnap::Kinds(m) => self.landmark_kind_overrides = m.clone(),
        }
        self.anno_undo.set_last_step(AnnoStep { store: s.store(), shifted: was != s.len() });
    }

    /// Runs `f` (a mutator of `store`) and records one ledger row for it **if
    /// the store changed**. `label` is the history row's words, computed by the
    /// caller **before** the call when the edit removes what it names (a delete).
    /// `stream` is the gesture key of a per-sample setter: it merges into the
    /// open gesture's entry (`AnnoUndo::begin_stream`), and is ignored outside one.
    /// `repoint` is a vault-note re-pointing the mutator itself performs, so undo can reverse it.
    pub(crate) fn anno_scope<R>(
        &mut self,
        store: AnnoStore,
        label: String,
        stream: Option<String>,
        repoint: Option<Repoint>,
        f: impl FnOnce(&mut WorldGen) -> R,
    ) -> R {
        self.anno_scope_multi(&[store], label, stream, repoint, f)
    }

    /// [`Self::anno_scope`] over several stores at once (`TOWN_LAYOUT_SAVE_SCOPE.md` TL-4: a town re-lay moves pins,
    /// labels and icons together): one ledger row whose main store is the first of `stores` that changed and whose
    /// `also` holds the rest that changed. **Every scope also watches the frozen-town table** (TL-3): a pin, label or
    /// icon placed or moved in a town freezes it inside the same step ([`Self::town_on_marks`]), so undoing the
    /// placement unfreezes the town it froze (scope §5's last row), and a step that changed only the table (a manual
    /// "Freeze this town") is still recorded.
    pub(crate) fn anno_scope_multi<R>(
        &mut self,
        stores: &[AnnoStore],
        label: String,
        stream: Option<String>,
        repoint: Option<Repoint>,
        f: impl FnOnce(&mut WorldGen) -> R,
    ) -> R {
        let pre: Vec<Option<AnnoSnap>> = stores.iter().map(|s| self.anno_snap(*s)).collect();
        let towns_pre = Arc::clone(&self.town_layouts);
        let r = f(self);
        let post: Vec<Option<AnnoSnap>> = stores.iter().map(|s| self.anno_snap(*s)).collect();
        // TL-3: marks that landed in a town freeze it (or anchor on its frozen layout), inside this same step.
        if !self.town_relaying {
            for (a, b) in pre.iter().zip(&post) {
                if let (Some(a), Some(b)) = (a, b) {
                    if a != b {
                        self.town_on_marks(a, b);
                    }
                }
            }
        }
        let towns = (!Arc::ptr_eq(&towns_pre, &self.town_layouts) && *towns_pre != *self.town_layouts)
            .then(|| (towns_pre, Arc::clone(&self.town_layouts)));
        let mut changed: Vec<(AnnoSnap, AnnoSnap)> = Vec::new();
        let mut first_any: Option<(AnnoSnap, AnnoSnap)> = None;
        for (a, b) in pre.into_iter().zip(post) {
            if let (Some(a), Some(b)) = (a, b) {
                if a != b {
                    changed.push((a, b));
                } else if first_any.is_none() {
                    first_any = Some((a, b));
                }
            }
        }
        if changed.is_empty() {
            // Only the town table moved (a manual freeze): recorded under the first store, unchanged.
            match (towns.is_some(), first_any) {
                (true, Some(p)) => changed.push(p),
                _ => return r,
            }
        }
        let (main_pre, main_post) = changed.remove(0);
        self.anno_record(main_pre.store(), label, stream, repoint, main_pre, main_post, changed, towns);
        r
    }

    /// Records one changed edit: merges it into the open gesture's entry when
    /// it is that gesture's next sample, else appends a new ledger row.
    /// A new row is a new operation: the height redo tail and every undone
    /// annotation go (an undone way drag dies on its ledger stamp).
    #[allow(clippy::too_many_arguments)]
    fn anno_record(
        &mut self,
        store: AnnoStore,
        label: String,
        stream: Option<String>,
        repoint: Option<Repoint>,
        pre: AnnoSnap,
        post: AnnoSnap,
        also: Vec<(AnnoSnap, AnnoSnap)>,
        towns: Option<(Arc<crate::town_layout_bridge::TownTable>, Arc<crate::town_layout_bridge::TownTable>)>,
    ) {
        let after = Arc::new(post);
        // Inside a compound every edit is the compound's, under its name and one shared key.
        let (label, key) = match self.anno_undo.compound_label() {
            Some(c) => (c.to_string(), Some(crate::annotation_undo::COMPOUND_KEY.to_string())),
            None => (label, stream.filter(|_| self.anno_undo.is_streaming())),
        };
        if let (Some(k), true) = (key.as_deref(), also.is_empty()) {
            if self.anno_undo.merge_stream(self.ledger.last_seq(), store, k, &pre, Arc::clone(&after), repoint.map(|r| r.to), towns.clone()) {
                return;
            }
        }
        // Entries the store moved out from under are dropped before this one is
        // held: judged against the world as it stood BEFORE this edit (the
        // `record_delete` rule, for the same reason -- pruning against the
        // post-edit store would drop every earlier entry of this store).
        let live = self.anno_undo.live_seqs(&|s| {
            if s.store() == store {
                *s == pre
            } else if let Some((b, _)) = also.iter().find(|(b, _)| b.store() == s.store()) {
                s == b
            } else {
                self.anno_matches(s)
            }
        });
        self.anno_undo.retain_seqs(&live);
        let before = self.anno_undo.shared_before(store, &pre);
        let seq = self.ledger.record("annotations", label.clone(), "", undo::EntryKind::Annotation);
        let also = also.into_iter().map(|(b, a)| (Arc::new(b), Arc::new(a))).collect();
        self.anno_undo.push(AnnoEntry { seq, label, store, before, after, stream: key, repoint, also, towns });
        self.anno_undo.note_recorded();
        self.redo.clear();
    }

    /// Re-points a note filed by an element's cell, both directions of an edit.
    fn anno_repoint(&mut self, r: Repoint, reverse: bool) {
        let (from, to) = if reverse { (r.to, r.from) } else { (r.from, r.to) };
        self.vault.store.reassign_entity(r.kind, from, to);
    }

    /// Every ledger `seq` that can still be put back, of the way and
    /// annotation kinds: the one set `undo_ledger` and `undo_revert_to` hand
    /// the ledger as its held-live closure.
    pub(crate) fn held_live_seqs(&self) -> Vec<u64> {
        let mut v = self.way_undo.live_seqs(&|s| self.way_fp(s));
        v.extend(self.anno_undo.live_seqs(&|s| self.anno_matches(s)));
        // F26: the newest sphere promotion, while it is still undoable. Only
        // the newest: an older one's faction is no longer the roster's last.
        if let (Some(u), Some(c)) = (self.sphere_undo.last(), self.civ.as_ref()) {
            if crate::city_state_bridge::promote_live(c, u) {
                v.push(u.seq);
            }
        }
        v
    }

    /// Reverts the newest live annotation row and returns its label. `None`
    /// when no entry is live. The state is written back, the note re-pointing
    /// reversed, the ledger row removed and the entry moved to the redo stack
    /// stamped with the height depth and the ledger position.
    pub(crate) fn undo_anno_step(&mut self) -> Option<String> {
        let seq = self.anno_undo.newest_live(&|s| self.anno_matches(s))?.seq;
        self.undo_anno_seq(seq)
    }

    /// [`Self::undo_anno_step`] for the one held entry `seq` (the caller has established it is live).
    pub(crate) fn undo_anno_seq(&mut self, seq: u64) -> Option<String> {
        let e = self.anno_undo.remove(seq)?;
        for (b, _) in &e.also {
            self.anno_apply(b);
        }
        if let Some((b, _)) = &e.towns {
            self.town_install(Arc::clone(b));
        }
        self.anno_apply(&e.before);
        if let Some(r) = e.repoint {
            self.anno_repoint(r, true);
        }
        self.ledger.remove_seq(seq);
        let label = e.label.clone();
        self.anno_undo.push_redo(AnnoRedo { id: undo::next_step_id(), entry: e, height_depth: self.undo.depth(), mark: self.ledger.last_seq() });
        Some(label)
    }

    /// The undone annotation edit a redo would re-apply, **only** while the
    /// world is exactly as its undo left it (see `AnnoUndo::redo_ready`).
    pub(crate) fn anno_redo_offer(&self) -> Option<(u64, String)> {
        self.anno_undo
            .redo_ready(self.ledger.last_seq(), self.undo.depth(), &|s| self.anno_matches(s))
            .map(|r| (r.id, r.entry.label.clone()))
    }

    /// The ready drag and annotation offers in the order `redo_last` takes
    /// them, newest undo first ([`order_offers`]). The height tail is not in
    /// this list: it comes after whatever is here.
    pub(crate) fn redo_offers(&self) -> Vec<RedoOffer> {
        let manual = self.infra.as_ref().map(|t| t.ways.as_slice());
        let drag = crate::context_pick_bridge::drag_redo_ready(&self.way_undo, &self.ledger, manual, self.undo.depth()).map(|r| (r.id, r.label.clone()));
        order_offers(drag, self.anno_redo_offer(), self.sphere_redo_offer())
    }

    /// [`Self::redo_offers`] plus the undone drags BEHIND a ready top drag
    /// ([`crate::context_pick_bridge::WayUndo::redo_chain_below_top`]), each placed right after it,
    /// so a history list counts every drag a run of redos will re-apply instead of the top one only.
    /// Listing only: `redo_last` still takes [`Self::redo_offers`]' first entry.
    pub(crate) fn redo_offers_listed(&self) -> Vec<RedoOffer> {
        let mut v = self.redo_offers();
        let Some(at) = v.iter().position(|o| o.kind == RedoKind::Drag) else { return v };
        let below = self.way_undo.redo_chain_below_top().into_iter().map(|r| RedoOffer { kind: RedoKind::Drag, id: r.id, label: r.label.clone() });
        let tail: Vec<RedoOffer> = v.split_off(at + 1);
        v.extend(below);
        v.extend(tail);
        v
    }

    /// Re-applies the top undone annotation edit if it is offered: writes its
    /// `after`, records a **new** ledger row (so `seq` keeps climbing, as
    /// `redo_last` does for height), re-holds the entry so it is undoable
    /// again, and re-stamps the redo entries that were valid until this record
    /// (both stacks). Returns the label, or `None` when nothing is offered.
    pub(crate) fn redo_anno_step(&mut self) -> Option<String> {
        self.anno_redo_offer()?;
        let top = self.anno_undo.pop_redo()?;
        let AnnoRedo { entry, .. } = top;
        for (_, a) in &entry.also {
            self.anno_apply(a);
        }
        if let Some((_, a)) = &entry.towns {
            self.town_install(Arc::clone(a));
        }
        self.anno_apply(&entry.after);
        if let Some(r) = entry.repoint {
            self.anno_repoint(r, false);
        }
        let before = self.ledger.last_seq();
        let seq = self.ledger.record("annotations", entry.label.clone(), "", undo::EntryKind::Annotation);
        let label = entry.label.clone();
        self.anno_undo.push(AnnoEntry { seq, stream: None, ..entry });
        self.anno_undo.restamp_redo(before, self.ledger.last_seq());
        self.way_undo.restamp_redo(before, self.ledger.last_seq());
        self.restamp_sphere_redo(before);
        Some(label)
    }
}

/// The three `#[func]`s of the annotation undo.
#[godot_api(secondary)]
impl WorldGen {
    /// Opens a gesture: until [`Self::annotation_stream_end`], per-sample setters
    /// (`label_move`, a label's size, angle or arc, `icon_resize`, an icon-brush
    /// stamp) merge into one undo step. Call it on the pointer press or slider
    /// drag start; the **first** sample after it is always a new step.
    #[func]
    fn annotation_stream_begin(&mut self) {
        self.anno_undo.begin_stream();
    }

    /// Closes the gesture opened by [`Self::annotation_stream_begin`]. Harmless when none is open.
    #[func]
    fn annotation_stream_end(&mut self) {
        self.anno_undo.end_stream();
    }

    /// Opens a compound named `label`: until [`Self::annotation_compound_end`] every
    /// edit of ONE store (a new label is a create, a reclass and a style set) folds
    /// into a single undo step carrying this name. Nothing is recorded when no edit
    /// follows. Not for edits of two stores: the second store's edits would start a new step.
    #[func]
    fn annotation_compound_begin(&mut self, label: GString) {
        self.anno_undo.begin_compound(&label.to_string());
    }

    /// Closes the compound opened by [`Self::annotation_compound_begin`]. Harmless when none is open.
    #[func]
    fn annotation_compound_end(&mut self) {
        self.anno_undo.end_compound();
    }

    /// What the last undo or redo of an annotation step did, for the shell's
    /// repaint: `{store: "label"|"icon"|"name"|"poi", shifted: bool, n: int}`. **Empty**
    /// before any such step (never a plausible store name for "none").
    /// `shifted` means the row count changed, so every later index moved and a
    /// selection or history entry naming one is stale.
    #[func]
    fn annotation_last_step(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        if let Some(s) = self.anno_undo.last_step() {
            d.set("store", s.store.key());
            d.set("shifted", s.shifted);
            // The running count, so a caller comparing it to the last number it saw can tell a
            // new step from the same one read twice (never reset, even across a generate).
            d.set("n", self.anno_undo.step_no() as i64);
        }
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the offer undone **later** (the larger step id) is redone
    /// first, whichever stack it is on, and a lone offer is returned as is.
    #[test]
    fn the_later_undo_is_redone_first() {
        let d = || Some((3u64, "Draw road".to_string()));
        let a = |id: u64| Some((id, "Move label".to_string()));
        let kinds = |v: Vec<RedoOffer>| v.into_iter().map(|o| o.kind).collect::<Vec<_>>();
        assert_eq!(kinds(order_offers(d(), a(4), None)), vec![RedoKind::Annotation, RedoKind::Drag]);
        assert_eq!(kinds(order_offers(d(), a(2), None)), vec![RedoKind::Drag, RedoKind::Annotation]);
        assert_eq!(kinds(order_offers(d(), None, None)), vec![RedoKind::Drag]);
        assert_eq!(kinds(order_offers(None, a(9), None)), vec![RedoKind::Annotation]);
        assert!(order_offers(None, None, None).is_empty());
        // F26: an undone sphere promotion orders with the others by step id, like any offer.
        let sp = |id: u64| Some((id, "Make Ash a faction".to_string()));
        assert_eq!(kinds(order_offers(d(), a(2), sp(9))), vec![RedoKind::Sphere, RedoKind::Drag, RedoKind::Annotation]);
        assert_eq!(kinds(order_offers(d(), None, sp(1))), vec![RedoKind::Drag, RedoKind::Sphere]);
    }
}

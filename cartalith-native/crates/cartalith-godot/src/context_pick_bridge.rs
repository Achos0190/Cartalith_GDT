//! CM-7's two engine rows (`MAP_CONTEXT_SCOPE.md` §9.2; Ruling BA,
//! 2026-09-28): the **way** pick with its undoable delete, and the **river**
//! pick resolved as "the branch to its mouth", with its catchment.
//!
//! Its own file with a `#[godot_api(secondary)]` block, like
//! `campaign_bridge.rs`. The picks are read-only; the one write is
//! [`WorldGen::way_delete`], and the undo it records is served by `lib.rs`'s
//! global undo block, which asks [`WorldGen::next_undo_step`] here which of
//! the two reversible kinds is newer.
//!
//! ## Ways: three stores, one list
//!
//! The reference keeps one flat `civWays` array -- the generated network, its
//! sea lanes and every hand-drawn way -- and its way list deletes any of them
//! with `civWays.splice(ri,1)` (v2.11 line 17688) and no recompute. This port
//! splits that array three ways ([`WayStore`]): `CivData::ways`,
//! `CivData::sea_routes` and `InfraTools::ways`. A pick answers which store and
//! which index; a delete removes exactly that entry, as the reference does.
//!
//! **A generated way is not protected from the next rebuild, and the card says
//! so.** Generate roads (`civ_auto_routes`), Auto-populate and a regenerate all
//! replace `CivData::ways` wholesale, so a deleted generated way comes back
//! with the next network rebuild. Hand-drawn ways survive a rebuild (the
//! reference's `civWays.filter(w => w.manual)`), so their delete sticks.
//!
//! ## The undo: a held way, validated by fingerprint
//!
//! The global undo stack snapshots the height field and nothing else
//! (`undo.rs`). A way delete is therefore recorded as its own reversible kind
//! ([`undo::EntryKind::WayDelete`]) whose payload -- the whole deleted entry
//! and its index -- lives in [`WayUndo`]. Restoring re-inserts it at the same
//! index, so endpoints, cells, type, name and `tid` all come back exactly.
//!
//! What makes that honest is **refusing when the store has moved**. Each entry
//! carries the store's fingerprint just before and just after its own delete;
//! an entry is live only while the store still matches (see
//! [`WayUndo::live_seqs`]). A rebuild, a regenerate or a Clear ways in between
//! changes the fingerprint, and the entry is then drawn in the history as not
//! reversible rather than re-inserting a way into a network it no longer
//! belongs to.
//!
//! ## Rivers: the branch to its mouth
//!
//! Rivers are unnamed in this engine (`right_dock.gd`'s River context carries
//! the reason: `naming::FeatureKind` has no river form), so a picked river is
//! described, never named. The branch is the channel receiver chain
//! (`ChannelResult::recv`, the tree the drawn rivers are traced along) from
//! the clicked cell downstream; where that chain stops on dry land it is
//! continued along the flow-accumulation tree (`flow_receivers` over the
//! routing surface, the tree `flow_discharge` was built on), until the first
//! sea or lake cell. A walk that still ends on land is reported as ending
//! inland (or at the map edge), not as reaching the sea.

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use cartalith_civ::tools::ManualWay;
use cartalith_civ::{SeaRoute, Way};
use godot::prelude::*;

use crate::{undo, WorldGen, WorldSource};

// ============================================================ way stores ==

/// Which of the three lists a way lives in. The reference's single `civWays`
/// array, split the way this port stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WayStore {
    /// `CivData::ways` -- the generated road network.
    Generated,
    /// `CivData::sea_routes` -- the generated sea lanes.
    SeaLane,
    /// `InfraTools::ways` -- hand-drawn with the Way tool (land or sea).
    Manual,
}

/// Wire-key conversion for [`WayStore`]; the two directions must stay inverse.
impl WayStore {
    /// Every store, in the fixed order `WayUndo::live_seqs` walks them.
    pub const ALL: [WayStore; 3] = [WayStore::Generated, WayStore::SeaLane, WayStore::Manual];

    /// The store's wire key as `way_pick`/`way_get` return it and `way_delete`
    /// accepts it. Must stay the exact inverse of [`WayStore::parse`].
    pub fn key(self) -> &'static str {
        match self {
            WayStore::Generated => "generated",
            WayStore::SeaLane => "sea_lane",
            WayStore::Manual => "manual",
        }
    }

    /// The inverse of [`WayStore::key`]; `None` for any other string, so a
    /// caller's typo cannot select a store.
    pub fn parse(s: &str) -> Option<WayStore> {
        match s {
            "generated" => Some(WayStore::Generated),
            "sea_lane" => Some(WayStore::SeaLane),
            "manual" => Some(WayStore::Manual),
            _ => None,
        }
    }
}

/// One deleted way, whole -- whichever store it came from -- or, for
/// [`HeldWay::Drag`], the whole set of hand-drawn ways one committed drag pushed.
#[derive(Debug, Clone, PartialEq)]
pub enum HeldWay {
    Generated(Way),
    SeaLane(SeaRoute),
    Manual(ManualWay),
    /// ET-D4: every way one drag commit pushed (one per symmetry copy), in push
    /// order. They are contiguous in `InfraTools::ways` starting at the
    /// entry's `index`. Unlike the other variants this is held **after a draw**,
    /// not after a delete: undoing it removes them and redoing it re-inserts them.
    Drag(Vec<ManualWay>),
}

/// Accessors for a held way.
impl HeldWay {
    /// Which store this held way must be put back into.
    pub fn store(&self) -> WayStore {
        match self {
            HeldWay::Generated(_) => WayStore::Generated,
            HeldWay::SeaLane(_) => WayStore::SeaLane,
            HeldWay::Manual(_) | HeldWay::Drag(_) => WayStore::Manual,
        }
    }
}

/// Mutable views of the three stores, `None` where the store does not exist
/// (no civ layer yet, no infra tools yet).
pub struct WayStores<'a> {
    /// The generated road network (`CivData::ways`); `None` before a civ layer exists.
    pub generated: Option<&'a mut Vec<Way>>,
    /// The generated sea lanes (`CivData::sea_routes`); `None` before a civ layer exists.
    pub sea: Option<&'a mut Vec<SeaRoute>>,
    /// The hand-drawn ways (`InfraTools::ways`); `None` before infra tools exist.
    pub manual: Option<&'a mut Vec<ManualWay>>,
}

/// Remove `index` from `store`. `None` for a missing store or an out-of-range
/// index -- nothing is removed then.
pub fn take_way(st: &mut WayStores<'_>, store: WayStore, index: usize) -> Option<HeldWay> {
    match store {
        WayStore::Generated => {
            let v = st.generated.as_deref_mut()?;
            (index < v.len()).then(|| HeldWay::Generated(v.remove(index)))
        }
        WayStore::SeaLane => {
            let v = st.sea.as_deref_mut()?;
            (index < v.len()).then(|| HeldWay::SeaLane(v.remove(index)))
        }
        WayStore::Manual => {
            let v = st.manual.as_deref_mut()?;
            (index < v.len()).then(|| HeldWay::Manual(v.remove(index)))
        }
    }
}

/// Put `way` back at `index` of its own store. `false` (and nothing written)
/// when the store is missing or `index` is past its end.
pub fn put_way(st: &mut WayStores<'_>, index: usize, way: HeldWay) -> bool {
    match way {
        HeldWay::Generated(w) => match st.generated.as_deref_mut() {
            Some(v) if index <= v.len() => {
                v.insert(index, w);
                true
            }
            _ => false,
        },
        HeldWay::SeaLane(w) => match st.sea.as_deref_mut() {
            Some(v) if index <= v.len() => {
                v.insert(index, w);
                true
            }
            _ => false,
        },
        HeldWay::Manual(w) => match st.manual.as_deref_mut() {
            Some(v) if index <= v.len() => {
                v.insert(index, w);
                true
            }
            _ => false,
        },
        // Re-inserts the whole drag contiguously at `index`, in push order: the
        // redo of a drag (`redo_drag_core`). The *undo* of a drag is not this
        // function's inverse -- `undo_way_core` removes the slice itself, after
        // checking it still holds exactly these ways.
        HeldWay::Drag(ws) => match st.manual.as_deref_mut() {
            Some(v) if index <= v.len() => {
                v.splice(index..index, ws);
                true
            }
            _ => false,
        },
    }
}

/// Feeds the fields every way kind shares into `h`: point count, first and
/// last point (as bit patterns, so no float comparison), breaks, length and
/// name. Deliberately not every vertex; see [`fp_generated`].
fn hash_pts(h: &mut DefaultHasher, pts: &[(f64, f64)], brks: &[usize], km: f64, name: &str) {
    pts.len().hash(h);
    if let (Some(a), Some(b)) = (pts.first(), pts.last()) {
        a.0.to_bits().hash(h);
        a.1.to_bits().hash(h);
        b.0.to_bits().hash(h);
        b.1.to_bits().hash(h);
    }
    brks.hash(h);
    km.to_bits().hash(h);
    name.hash(h);
}

/// A cheap identity for one store's current contents: length plus, per way,
/// its point count, end points, breaks, length, name and type. **Not** a hash
/// of every vertex -- it only has to change when the list is rebuilt, cleared
/// or edited under a held undo entry, and every one of those moves at least a
/// length, an end point or a count. `None` (no store) hashes to its own value.
pub fn fp_generated(ways: Option<&[Way]>) -> u64 {
    let mut h = DefaultHasher::new();
    0u8.hash(&mut h);
    match ways {
        None => u64::MAX.hash(&mut h),
        Some(ws) => {
            ws.len().hash(&mut h);
            for w in ws {
                hash_pts(&mut h, &w.pts, &w.brks, w.km, &w.name);
                (w.way_type as u8).hash(&mut h);
                (w.tid, w.a_idx, w.b_idx, w.hidden).hash(&mut h);
            }
        }
    }
    h.finish()
}

/// Fingerprint of the sea-lane store (domain tag 1), same contract as
/// [`fp_generated`]. Sea lanes carry no type or flags, so only the shared
/// fields are hashed.
pub fn fp_sea(lanes: Option<&[SeaRoute]>) -> u64 {
    let mut h = DefaultHasher::new();
    1u8.hash(&mut h);
    match lanes {
        None => u64::MAX.hash(&mut h),
        Some(ls) => {
            ls.len().hash(&mut h);
            for w in ls {
                hash_pts(&mut h, &w.pts, &w.brks, w.km, &w.name);
            }
        }
    }
    h.finish()
}

/// Fingerprint of the hand-drawn store (domain tag 2), same contract as
/// [`fp_generated`], plus the way type, the sea flag and the hidden flag.
pub fn fp_manual(ways: Option<&[ManualWay]>) -> u64 {
    let mut h = DefaultHasher::new();
    2u8.hash(&mut h);
    match ways {
        None => u64::MAX.hash(&mut h),
        Some(ws) => {
            ws.len().hash(&mut h);
            for w in ws {
                hash_pts(&mut h, &w.pts, &w.brks, w.km, &w.name);
                (w.way_type as u8, w.sea, w.hidden).hash(&mut h);
            }
        }
    }
    h.finish()
}

// ============================================================== way undo ==

/// One undoable way delete: the ledger row's `seq`, where the way was, the way
/// itself, and its store's fingerprint either side of the delete.
#[derive(Debug, Clone, PartialEq)]
pub struct WayUndoEntry {
    /// The ledger row's sequence number this held way belongs to.
    pub seq: u64,
    /// The way's index in its store at deletion; where undo re-inserts it.
    pub index: usize,
    /// The whole deleted entry, so every field comes back on undo.
    pub way: HeldWay,
    /// The store's fingerprint just before this delete.
    pub fp_before: u64,
    /// The store's fingerprint just after this delete; the entry is live only
    /// while the store still matches it (or the chain above it).
    pub fp_after: u64,
}

/// How many deletes are held (a judgement, not a measurement). A held way is a few hundred points at most, so
/// this is a sanity bound, not a memory budget; the ledger's own
/// [`undo::MAX_LEDGER`] is larger.
pub const MAX_WAY_UNDO: usize = 64;

/// One undone drag, waiting to be redone (ET-D4). Held by [`WayUndo`] on a LIFO
/// stack; only the **top** can be redone, and only while [`drag_redo_ready`]
/// says the world is exactly as the undo left it.
#[derive(Debug, Clone, PartialEq)]
pub struct DragRedo {
    /// Index in `InfraTools::ways` where the drag's first way sat (and sits again).
    pub index: usize,
    /// The drag's ways, whole, in push order -- what the redo re-inserts.
    pub ways: Vec<ManualWay>,
    /// The manual store's fingerprint as the undo **left** it (= the drag's own
    /// `fp_before`). The redo is offered only while the store still has it.
    pub fp_before: u64,
    /// The store's fingerprint with the drag present, carried for the redone entry.
    pub fp_after: u64,
    /// The history label (`Draw road`), reused for the redone ledger row.
    pub label: String,
    /// `undo.depth()` when the drag was undone: redo is offered only at the same
    /// height-stack depth, which is what orders it against the height redo tail
    /// (see `redo_last`).
    pub height_depth: usize,
    /// `HistoryLedger::last_seq()` at the undo, re-stamped by the redos that
    /// follow it. A commit since moves the ledger past it and kills the offer: a
    /// new operation truncates redo, as everywhere else in this ledger.
    pub mark: u64,
}

/// The held side of every [`undo::EntryKind::WayDelete`] and
/// [`undo::EntryKind::WayDrag`] ledger row, plus the drag redo stack.
#[derive(Debug, Default)]
pub struct WayUndo {
    entries: Vec<WayUndoEntry>,
    /// Undone drags, newest undo last (the top). Cleared with `entries`, so a
    /// generate/load/`clear_undo` drops it exactly as it drops the undo side.
    redo: Vec<DragRedo>,
}

/// The held-way list and its liveness rule ([`WayUndo::live_seqs`]).
impl WayUndo {
    /// An empty held-way list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes one undone drag onto the redo stack, dropping the oldest past
    /// [`MAX_WAY_UNDO`].
    pub fn push_redo(&mut self, r: DragRedo) {
        self.redo.push(r);
        if self.redo.len() > MAX_WAY_UNDO {
            self.redo.remove(0);
        }
    }

    /// The drag a redo would re-apply next (not yet validated: see [`drag_redo_ready`]).
    pub fn redo_top(&self) -> Option<&DragRedo> {
        self.redo.last()
    }

    /// Takes the top redo entry off the stack.
    pub fn pop_redo(&mut self) -> Option<DragRedo> {
        self.redo.pop()
    }

    /// Forgets every undone drag (a new operation was committed).
    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }

    /// How many undone drags are held (raw occupancy, not availability).
    #[cfg(test)]
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// After a legitimate redo recorded its own ledger row, moves every redo
    /// entry stamped `old` (the ledger's `last_seq` just before that record) to
    /// `new`. Entries stamped earlier stay stamped earlier -- a commit happened
    /// after *their* undo, so they must stay dead.
    pub fn restamp_redo(&mut self, old: u64, new: u64) {
        for r in self.redo.iter_mut().filter(|r| r.mark == old) {
            r.mark = new;
        }
    }

    /// The newest live entry of one store, per [`WayUndo::live_seqs`].
    pub fn newest_live_in(&self, store: WayStore, fp: &dyn Fn(WayStore) -> u64) -> Option<&WayUndoEntry> {
        let seq = self.live_seqs(fp).into_iter().find(|s| self.entries.iter().any(|e| e.seq == *s && e.way.store() == store))?;
        self.entries.iter().find(|e| e.seq == seq)
    }

    /// Holds one more deleted way, dropping the oldest past [`MAX_WAY_UNDO`].
    pub fn push(&mut self, e: WayUndoEntry) {
        self.entries.push(e);
        if self.entries.len() > MAX_WAY_UNDO {
            self.entries.remove(0);
        }
    }

    /// Drops every held way and every undone drag. Their ledger rows are the caller's to handle.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.redo.clear();
    }

    /// How many entries are held (test-only; production asks `live_seqs`).
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Every entry that can still be put back, newest first.
    ///
    /// Per store, walking newest to oldest: the newest entry is live when the
    /// store's current fingerprint is the one it left behind (`fp_after`);
    /// each older entry is live when it left behind exactly what the entry
    /// above it found (`fp_before`). The first mismatch ends that store's
    /// chain -- something else changed the list there, and nothing at or
    /// below it can be re-inserted into the list as it now stands.
    pub fn live_seqs(&self, fp: &dyn Fn(WayStore) -> u64) -> Vec<u64> {
        let mut out = Vec::new();
        for store in WayStore::ALL {
            let mut expected = fp(store);
            for e in self.entries.iter().rev().filter(|e| e.way.store() == store) {
                if e.fp_after != expected {
                    break;
                }
                out.push(e.seq);
                expected = e.fp_before;
            }
        }
        out.sort_unstable_by(|a, b| b.cmp(a));
        out
    }

    /// The newest entry that can still be put back, per [`WayUndo::live_seqs`].
    pub fn newest_live(&self, fp: &dyn Fn(WayStore) -> u64) -> Option<&WayUndoEntry> {
        let seq = *self.live_seqs(fp).first()?;
        self.entries.iter().find(|e| e.seq == seq)
    }

    /// Drop every entry that is no longer live. Their ledger rows stay, drawn
    /// as not reversible with [`STALE_REASON`].
    pub fn prune_stale(&mut self, fp: &dyn Fn(WayStore) -> u64) {
        let live = self.live_seqs(fp);
        self.entries.retain(|e| live.contains(&e.seq));
    }

    /// Takes the entry for ledger row `seq` out of the list; `None` if absent.
    pub fn remove(&mut self, seq: u64) -> Option<WayUndoEntry> {
        let pos = self.entries.iter().position(|e| e.seq == seq)?;
        Some(self.entries.remove(pos))
    }
}

/// The history row's reason for a way delete that can no longer be undone.
pub const STALE_REASON: &str =
    "the way list was rebuilt, cleared or edited since, so the deleted way would not fit back into it";

/// Which kind a plain `Edit ▸ Undo` reverts next.
#[derive(Debug, Clone, PartialEq)]
pub enum UndoPick {
    /// A height-field snapshot is newer; carries its label.
    Height(String),
    /// A live way delete is newer; carries its ledger `seq` and label.
    Way { seq: u64, label: String },
}

/// The unified choice, as a pure function: the newer of the newest height row
/// (when a height snapshot is held) and the newest live way row. `height_seq`
/// is `Some(0)` when a snapshot is held but no ledger row names it (a push
/// with no row -- the ledger is bounded), which any way row outranks.
pub fn choose_undo(height: Option<(u64, String)>, way: Option<(u64, String)>) -> Option<UndoPick> {
    match (height, way) {
        (None, None) => None,
        (Some((_, l)), None) => Some(UndoPick::Height(l)),
        (None, Some((seq, label))) => Some(UndoPick::Way { seq, label }),
        (Some((hs, l)), Some((ws, label))) => {
            if ws > hs {
                Some(UndoPick::Way { seq: ws, label })
            } else {
                Some(UndoPick::Height(l))
            }
        }
    }
}

// ====================================================== way undo / redo core ==
//
// The undo and redo of a way row as **pure functions** over the ledger, the held
// entries and the stores, so the semantics are unit-testable without a live
// Godot object (`WorldGen` is a cdylib `GodotClass`; see `MISTAKES.md`'s
// preflight row on probes). `WorldGen::undo_way_step`, `redo_last` and
// `way_stroke_end` are thin wrappers that hand these the real fields.
//
// ## Semantics of a drag row (ET-D4), decided and stated once
//
// * **Record.** A committed drag pushes exactly one [`undo::EntryKind::WayDrag`]
//   row, labelled `Draw <type>` (`Draw road`, `Draw sea lane`; `Draw road (2
//   ways)` when symmetry made copies), and holds its ways + start index + the
//   manual store's fingerprint either side. A refused or empty drag pushes nothing.
// * **Liveness.** The same fingerprint chain a way delete uses: the row is live
//   only while the manual store is exactly as the drag left it (or as the live
//   entries above it leave it). *Any* other change to the manual list -- a way
//   committed by clicking, a rename, a reload -- ends the chain: the row stays
//   in the history, drawn not reversible with [`STALE_REASON`], and nothing is
//   ever removed on its behalf. A way deleted **through the ledger** is not such
//   a change: its own row sits above the drag's and is undone first, after which
//   the drag is live again (the chain is walked, not broken).
// * **Undo** removes exactly the drag's ways (`index..index + n`), first
//   re-checking that slice equals the held ways value for value (the
//   fingerprint is a hash of selected fields, not a proof). On any mismatch it
//   refuses: nothing is touched, the held entry is dropped, the row goes stale.
// * **Redo** is a **LIFO stack of undone drags**, offered only for the top
//   entry and only while [`drag_redo_ready`]: nothing recorded since the undo
//   (ledger stamp), the height stack at the same depth, and the manual store
//   exactly as the undo left it (fingerprint). It re-inserts the same ways at the
//   same index, records a **new** row (new `seq`, as `redo_last` does for height)
//   and re-holds the entry, so the drag is undoable again.

/// The ledger label of a committed drag: `Draw <type>`, with the way count when
/// symmetry made more than one copy. Never invents a type: an empty list is the
/// caller's bug and gets the neutral `Draw way`.
pub fn drag_label(ways: &[ManualWay]) -> String {
    let Some(first) = ways.first() else { return "Draw way".to_string() };
    let kind = crate::infra_tools_bridge::way_type_key(first.way_type).replace('_', " ");
    if ways.len() > 1 { format!("Draw {kind} ({} ways)", ways.len()) } else { format!("Draw {kind}") }
}

/// The ledger row's detail line for a drag of `n` ways.
fn drag_detail(n: usize) -> String {
    if n > 1 { format!("hand-drawn drag, {n} symmetry copies") } else { "hand-drawn drag".to_string() }
}

/// Records one **committed** drag: the manual store already holds
/// `manual[first..first + count]`, pushed by `way_drag_commit`. Returns the new
/// ledger row's `seq`, or `None` -- recording **nothing** -- when `count` is `0`
/// or the range is not inside the store (a refused drag never reaches this with a
/// real range).
///
/// `fp_before` is the manual store's fingerprint *before* the commit; the
/// fingerprint after is computed here. A new operation, so the drag redo stack is
/// dropped; stale held entries are pruned (`fps` = every store's fingerprint now).
/// The caller also clears the **height** redo tail, which this module cannot reach.
pub fn record_drag(
    ledger: &mut undo::HistoryLedger,
    wu: &mut WayUndo,
    manual: &[ManualWay],
    first: usize,
    count: usize,
    fp_before: u64,
    fps: &dyn Fn(WayStore) -> u64,
) -> Option<u64> {
    let end = first.checked_add(count)?;
    if count == 0 || end > manual.len() {
        return None;
    }
    let ways = manual[first..end].to_vec();
    let label = drag_label(&ways);
    let seq = ledger.record("civ", label, drag_detail(count), undo::EntryKind::WayDrag);
    wu.clear_redo();
    // Prune against the world as it stood BEFORE this commit (the manual store
    // at `fp_before`): an older entry is stale only if something other than a
    // ledgered row changed the list under it. Pruning against the post-commit
    // fingerprints would drop every earlier entry, since this very drag moved the
    // manual store past what each of them left behind.
    let pre_commit = |s: WayStore| if s == WayStore::Manual { fp_before } else { fps(s) };
    wu.prune_stale(&pre_commit);
    wu.push(WayUndoEntry { seq, index: first, way: HeldWay::Drag(ways), fp_before, fp_after: fp_manual(Some(manual)) });
    Some(seq)
}

/// Records one **completed** way delete: the way has already been taken out of
/// `store` at `index` (by `take_way`), `fp_before`/`fp_after` are that store's
/// fingerprints either side of the removal, and `fps` gives every store's
/// fingerprint **now** (after the removal). Records the `WayDelete` ledger row,
/// drops the drag redo stack (a new operation), prunes stale held entries and
/// holds this one. Returns the new row's `seq`. `WorldGen::way_delete` is the thin
/// wrapper that hands it the real stores.
///
/// **Why the prune reads `fp_before` for the deleted store** (found-bug sweep
/// 2026-10-11, R1). An older held entry of the same store is stale only if
/// something *other than a ledgered row* changed the list under it. This delete
/// is itself about to be held, so the chain must be checked against the world as
/// it stood just before it -- exactly [`record_drag`]'s rule. Pruning against the
/// post-delete fingerprint dropped every earlier entry of that store (the delete
/// had just moved the store past what each of them left behind), so a second
/// delete in a row made the first one un-undoable.
///
/// Must never prune against `fp_after` for `store`, and must never record a row
/// for a delete that did not happen (the caller only calls it after `take_way`
/// returned a way).
#[allow(clippy::too_many_arguments)]
pub fn record_delete(
    ledger: &mut undo::HistoryLedger,
    wu: &mut WayUndo,
    store: WayStore,
    index: usize,
    way: HeldWay,
    fp_before: u64,
    fp_after: u64,
    fps: &dyn Fn(WayStore) -> u64,
) -> u64 {
    // A delete is a new operation: an undone drag can no longer be redone.
    wu.clear_redo();
    let title = held_way_title(&way);
    let detail = match store {
        WayStore::Generated => "generated network -- Generate roads rebuilds it",
        WayStore::SeaLane => "generated sea lane -- Generate roads rebuilds it",
        WayStore::Manual => "hand-drawn",
    };
    let seq = ledger.record("civ", format!("Delete {title}"), detail, undo::EntryKind::WayDelete);
    let pre_delete = |s: WayStore| if s == store { fp_before } else { fps(s) };
    wu.prune_stale(&pre_delete);
    wu.push(WayUndoEntry { seq, index, way, fp_before, fp_after });
    seq
}

/// The history label for a held entry: `Delete <title>` for a deleted way, the
/// drag's own `Draw ...` label for a drag.
pub fn way_entry_label(e: &WayUndoEntry) -> String {
    match &e.way {
        HeldWay::Drag(ways) => drag_label(ways),
        w => format!("Delete {}", held_way_title(w)),
    }
}

/// What one way undo did: the label of the reverted row and whether it was a drag
/// (so the caller can drop `InfraTools`' own single-level drag record).
#[derive(Debug, Clone, PartialEq)]
pub struct WayStep {
    /// The reverted row's history label.
    pub label: String,
    /// `true` when the reverted row was a drag.
    pub was_drag: bool,
}

/// Reverts the newest live way row (delete **or** drag). `None` when there is no
/// live entry or the revert was refused. See [`undo_way_core_at`]. Test-only: the
/// `WorldGen` wrappers pick the `seq` themselves (`undo_way_step`,
/// `way_stroke_undo`) and call [`undo_way_core_at`]; this is that same pick-then-call
/// in one place for the harness.
#[cfg(test)]
pub fn undo_way_core(
    ledger: &mut undo::HistoryLedger,
    wu: &mut WayUndo,
    st: &mut WayStores<'_>,
    fp: &dyn Fn(WayStore) -> u64,
    height_depth: usize,
) -> Option<WayStep> {
    let seq = wu.newest_live(fp)?.seq;
    undo_way_core_at(ledger, wu, st, seq, height_depth)
}

/// Reverts the held entry `seq` (the caller has established it is live): a
/// deleted way is put back at its index; a drag has exactly its ways **removed**
/// and is pushed onto the redo stack, stamped with `height_depth` and the
/// ledger's current `last_seq`. The ledger row goes in both cases (the state it
/// recorded is no longer in effect). A refusal -- store missing, slice no longer
/// equal to the held ways, index past the end -- writes nothing, drops the held
/// entry (its row then reads as stale) and returns `None`: never a panic, never a
/// partial edit.
pub fn undo_way_core_at(
    ledger: &mut undo::HistoryLedger,
    wu: &mut WayUndo,
    st: &mut WayStores<'_>,
    seq: u64,
    height_depth: usize,
) -> Option<WayStep> {
    let e = wu.remove(seq)?;
    let label = way_entry_label(&e);
    let WayUndoEntry { index, way, fp_before, fp_after, .. } = e;
    match way {
        HeldWay::Drag(ways) => {
            let n = ways.len();
            let v = st.manual.as_deref_mut()?;
            let end = index.checked_add(n)?;
            if n == 0 || v.get(index..end) != Some(&ways[..]) {
                return None;
            }
            v.drain(index..end);
            ledger.remove_seq(seq);
            wu.push_redo(DragRedo {
                index,
                ways,
                fp_before,
                fp_after,
                label: label.clone(),
                height_depth,
                mark: ledger.last_seq(),
            });
            Some(WayStep { label, was_drag: true })
        }
        other => {
            let ok = put_way(st, index, other);
            ledger.remove_seq(seq);
            ok.then_some(WayStep { label, was_drag: false })
        }
    }
}

/// The drag a redo would re-apply, **only** when the world is exactly as that
/// drag's undo left it: the top of the redo stack, its ledger stamp equal to the
/// ledger's `last_seq` (nothing recorded since), the height stack at the depth
/// it had, and the manual store's fingerprint equal to the one the undo left.
/// `None` otherwise -- a stale offer is never made.
pub fn drag_redo_ready<'a>(
    wu: &'a WayUndo,
    ledger: &undo::HistoryLedger,
    manual: Option<&[ManualWay]>,
    height_depth: usize,
) -> Option<&'a DragRedo> {
    let top = wu.redo_top()?;
    (top.mark == ledger.last_seq() && top.height_depth == height_depth && fp_manual(manual) == top.fp_before).then_some(top)
}

/// Re-applies the top undone drag if [`drag_redo_ready`]: re-inserts its ways
/// at the same index, records a **new** `WayDrag` row (so `seq` keeps climbing),
/// re-holds the entry so the drag is undoable again, and re-stamps the redo
/// entries that were valid until this record. Returns the label, or `None`
/// (nothing changed except that an unplaceable top entry is dropped).
pub fn redo_drag_core(
    ledger: &mut undo::HistoryLedger,
    wu: &mut WayUndo,
    manual: Option<&mut Vec<ManualWay>>,
    height_depth: usize,
) -> Option<String> {
    let v = manual?;
    let top = drag_redo_ready(wu, ledger, Some(v.as_slice()), height_depth)?.clone();
    wu.pop_redo();
    let n = top.ways.len();
    let mut st = WayStores { generated: None, sea: None, manual: Some(v) };
    if !put_way(&mut st, top.index, HeldWay::Drag(top.ways.clone())) {
        return None;
    }
    let before = ledger.last_seq();
    let seq = ledger.record("civ", top.label.clone(), drag_detail(n), undo::EntryKind::WayDrag);
    let fp_after = fp_manual(st.manual.as_deref().map(|m| m.as_slice()));
    wu.push(WayUndoEntry { seq, index: top.index, way: HeldWay::Drag(top.ways), fp_before: top.fp_before, fp_after });
    wu.restamp_redo(before, ledger.last_seq());
    Some(top.label)
}

// ============================================================== way pick ==

/// The nearest polyline to `p` within `tol`: `(candidate index, distance,
/// nearest point)`. `brks` are indices where a new stroke starts
/// (`map_overlay.gd` draws `[start, cut)` then starts again at `cut`), so the
/// segment from `cut - 1` to `cut` is not drawn and is not hit. A run of one
/// point is hit at that point. Ties keep the earlier candidate.
pub fn nearest_polyline(cands: &[(&[(f64, f64)], &[usize])], p: (f64, f64), tol: f64) -> Option<(usize, f64, (f64, f64))> {
    let mut best: Option<(usize, f64, (f64, f64))> = None;
    for (ci, (pts, brks)) in cands.iter().enumerate() {
        if pts.is_empty() {
            continue;
        }
        let mut consider = |d: f64, at: (f64, f64)| {
            if d <= tol && best.is_none_or(|b| d < b.1) {
                best = Some((ci, d, at));
            }
        };
        if pts.len() == 1 {
            let d = ((p.0 - pts[0].0).powi(2) + (p.1 - pts[0].1).powi(2)).sqrt();
            consider(d, pts[0]);
            continue;
        }
        for k in 1..pts.len() {
            if brks.contains(&k) {
                continue;
            }
            let (d, at) = point_segment(p, pts[k - 1], pts[k]);
            consider(d, at);
        }
    }
    best
}

/// Distance from `p` to the segment `a`-`b`, and the nearest point on it.
pub fn point_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> (f64, (f64, f64)) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    let at = (a.0 + t * dx, a.1 + t * dy);
    (((p.0 - at.0).powi(2) + (p.1 - at.1).powi(2)).sqrt(), at)
}

/// `get_roads()`' own type vocabulary for a generated way.
pub fn generated_way_type_key(t: cartalith_civ::WayType) -> &'static str {
    match t {
        cartalith_civ::WayType::Highway => "highway",
        cartalith_civ::WayType::Regional => "regional",
        cartalith_civ::WayType::Road => "road",
        cartalith_civ::WayType::Track => "track",
        cartalith_civ::WayType::Ancient => "ancient",
    }
}

// ============================================================ river pick ==

/// Where a branch's walk stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminus {
    /// The last cell is sea (water class 1).
    Sea,
    /// The last cell is a lake (water class 2).
    Lake,
    /// No receiver, on land, away from the edge: a closed basin.
    Inland,
    /// No receiver, on the non-wrapping border.
    MapEdge,
}

/// Wire-key conversion for [`Terminus`].
impl Terminus {
    /// The key `river_pick` returns as `terminus`; the shell matches on these
    /// exact strings.
    pub fn key(self) -> &'static str {
        match self {
            Terminus::Sea => "sea",
            Terminus::Lake => "lake",
            Terminus::Inland => "inland",
            Terminus::MapEdge => "edge",
        }
    }
}

/// The nearest channel cell (`order >= 1`) whose centre lies within `radius`
/// of `(gx, gy)`, with its distance. `x` wraps in `world` mode. Ties go to
/// the lower cell index.
#[allow(clippy::too_many_arguments)]
pub fn nearest_channel_cell(order: &[i16], gw: usize, gh: usize, world: bool, gx: f64, gy: f64, radius: f64) -> Option<(usize, f64)> {
    if gw == 0 || gh == 0 || order.len() < gw * gh || !(radius >= 0.0) {
        return None;
    }
    let r = radius.ceil() as i64 + 1;
    let (cx, cy) = (gx.floor() as i64, gy.floor() as i64);
    let mut best: Option<(usize, f64)> = None;
    for y in (cy - r).max(0)..=(cy + r).min(gh as i64 - 1) {
        for xr in cx - r..=cx + r {
            let x = if world {
                xr.rem_euclid(gw as i64)
            } else if xr < 0 || xr >= gw as i64 {
                continue;
            } else {
                xr
            };
            let i = y as usize * gw + x as usize;
            if order[i] < 1 {
                continue;
            }
            let mut dx = (x as f64 + 0.5 - gx).abs();
            if world {
                dx = dx.min(gw as f64 - dx);
            }
            let dy = y as f64 + 0.5 - gy;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= radius && best.is_none_or(|b| d < b.1 || (d == b.1 && i < b.0)) {
                best = Some((i, d));
            }
        }
    }
    best
}

/// Ruling BA's "branch to its mouth": `start`, then each cell's channel
/// receiver (`chan_recv`), falling back to `cont` (the flow-accumulation
/// receiver) where the channel chain stops, until the first cell whose
/// `water` class is sea (1) or lake (2). `start` itself is never tested for
/// water: a click on a lake reach is still a river cell. Bounded by the grid's
/// own cell count, so a malformed receiver tree cannot loop forever.
pub fn trace_branch(
    start: usize,
    gw: usize,
    gh: usize,
    world: bool,
    chan_recv: &[i32],
    cont: &mut dyn FnMut(usize) -> i64,
    water: &dyn Fn(usize) -> u8,
) -> (Vec<usize>, Terminus) {
    let n = gw * gh;
    let mut cells = vec![start];
    let mut cur = start;
    for _ in 0..n {
        let c = chan_recv.get(cur).copied().unwrap_or(-1) as i64;
        let next = if c >= 0 { c } else { cont(cur) };
        if next < 0 || next as usize >= n || next as usize == cur {
            let (x, y) = (cur % gw, cur / gw);
            let edge = y == 0 || y + 1 == gh || (!world && (x == 0 || x + 1 == gw));
            return (cells, if edge { Terminus::MapEdge } else { Terminus::Inland });
        }
        let next = next as usize;
        cells.push(next);
        match water(next) {
            1 => return (cells, Terminus::Sea),
            2 => return (cells, Terminus::Lake),
            _ => {}
        }
        cur = next;
    }
    (cells, Terminus::Inland)
}

/// The branch's length in cells: summed centre-to-centre steps, `x` wrapping
/// in `world` mode (a step across the seam is one cell, not `gw`).
pub fn branch_cells_length(cells: &[usize], gw: usize, world: bool) -> f64 {
    cells
        .windows(2)
        .map(|w| {
            let (ax, ay) = ((w[0] % gw) as f64, (w[0] / gw) as f64);
            let (bx, by) = ((w[1] % gw) as f64, (w[1] / gw) as f64);
            let mut dx = (bx - ax).abs();
            if world {
                dx = dx.min(gw as f64 - dx);
            }
            (dx * dx + (by - ay).powi(2)).sqrt()
        })
        .sum()
}

/// The one tree both river rows walk: a cell's channel receiver where it has
/// one (`chan[i] >= 0`), its flow-accumulation receiver (`d8[i]`) otherwise --
/// [`trace_branch`]'s own step, for every cell at once.
pub fn combined_receivers(chan: &[i32], d8: &[i32]) -> Vec<i32> {
    d8.iter()
        .enumerate()
        .map(|(i, &d)| match chan.get(i) {
            Some(&c) if c >= 0 => c,
            _ => d,
        })
        .collect()
}

/// Every cell whose receiver chain in `recv` reaches `target`, `target`
/// included: `1` in the mask, `0` elsewhere. A reversed-tree breadth-first
/// walk, O(cells).
pub fn catchment_mask(recv: &[i32], target: usize) -> Vec<u8> {
    let n = recv.len();
    let mut mask = vec![0u8; n];
    if target >= n {
        return mask;
    }
    let mut start = vec![0u32; n + 1];
    for &r in recv {
        if r >= 0 && (r as usize) < n {
            start[r as usize + 1] += 1;
        }
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut fill = start.clone();
    let mut kids = vec![0u32; start[n] as usize];
    for (i, &r) in recv.iter().enumerate() {
        if r >= 0 && (r as usize) < n {
            kids[fill[r as usize] as usize] = i as u32;
            fill[r as usize] += 1;
        }
    }
    let mut q = VecDeque::from([target]);
    mask[target] = 1;
    while let Some(c) = q.pop_front() {
        for &k in &kids[start[c] as usize..start[c + 1] as usize] {
            let k = k as usize;
            if mask[k] == 0 {
                mask[k] = 1;
                q.push_back(k);
            }
        }
    }
    mask
}

// ================================================================ WorldGen ==

/// The crate-internal half: undo fingerprints and the way-undo step, which
/// `lib.rs`'s global undo block calls. None of it is exposed to GDScript.
impl WorldGen {
    /// The current fingerprint of one way store -- [`WayUndo`]'s liveness key.
    pub(crate) fn way_fp(&self, store: WayStore) -> u64 {
        match store {
            WayStore::Generated => fp_generated(self.civ.as_ref().map(|c| c.ways.as_slice())),
            WayStore::SeaLane => fp_sea(self.civ.as_ref().map(|c| c.sea_routes.as_slice())),
            WayStore::Manual => fp_manual(self.infra.as_ref().map(|t| t.ways.as_slice())),
        }
    }

    /// What a plain `Edit ▸ Undo` reverts next -- see [`choose_undo`].
    pub(crate) fn next_undo_step(&self) -> Option<UndoPick> {
        let height = self.undo.next_label().map(|l| (self.ledger.newest_height_seq().unwrap_or(0), l.to_string()));
        let way = self.way_undo.newest_live(&|s| self.way_fp(s)).map(|e| (e.seq, self.way_undo_label(e)));
        choose_undo(height, way)
    }

    /// The history label for a held entry -- see [`way_entry_label`].
    fn way_undo_label(&self, e: &WayUndoEntry) -> String {
        way_entry_label(e)
    }

    /// Reverts the newest live way row (a delete **or** a drag) and returns its
    /// label. `None` when there is no live entry, or when the revert was refused
    /// (see [`undo_way_core_at`]: the refusal is the guard, not a path).
    pub(crate) fn undo_way_step(&mut self) -> Option<String> {
        let fps: Vec<(WayStore, u64)> = WayStore::ALL.iter().map(|&s| (s, self.way_fp(s))).collect();
        let fp = |s: WayStore| fps.iter().find(|(k, _)| *k == s).map_or(0, |(_, v)| *v);
        let seq = self.way_undo.newest_live(&fp)?.seq;
        self.undo_way_seq(seq).map(|s| s.label)
    }

    /// Reverts the one held entry `seq` (the caller has established it is live)
    /// through [`undo_way_core_at`], handing it the real stores, ledger and held
    /// list. After a drag it also drops `InfraTools`' own single-level drag
    /// record, so the ledger is the only route that can undo it.
    pub(crate) fn undo_way_seq(&mut self, seq: u64) -> Option<WayStep> {
        let depth = self.undo.depth();
        let step = {
            let (civ, infra) = (self.civ.as_mut(), self.infra.as_mut());
            let (generated, sea) = match civ {
                Some(c) => (Some(&mut c.ways), Some(&mut c.sea_routes)),
                None => (None, None),
            };
            let mut st = WayStores { generated, sea, manual: infra.map(|t| &mut t.ways) };
            undo_way_core_at(&mut self.ledger, &mut self.way_undo, &mut st, seq, depth)
        };
        if step.as_ref().is_some_and(|s| s.was_drag) {
            if let Some(t) = self.infra.as_mut() {
                t.way_drag_forget();
            }
        }
        step
    }

    /// The label of the drag a `redo_last()` would re-apply, or `None` -- see
    /// [`drag_redo_ready`] for exactly when one is offered.
    pub(crate) fn drag_redo_label(&self) -> Option<String> {
        let manual = self.infra.as_ref().map(|t| t.ways.as_slice());
        drag_redo_ready(&self.way_undo, &self.ledger, manual, self.undo.depth()).map(|r| r.label.clone())
    }

    /// Re-applies the top undone drag ([`redo_drag_core`]) against the real
    /// manual store. `None` when none is offered.
    pub(crate) fn redo_drag_step(&mut self) -> Option<String> {
        let depth = self.undo.depth();
        let label = redo_drag_core(&mut self.ledger, &mut self.way_undo, self.infra.as_mut().map(|t| &mut t.ways), depth)?;
        if let Some(t) = self.infra.as_mut() {
            t.way_drag_forget();
        }
        Some(label)
    }

    /// Records the drag `way_stroke_end` just committed as one ledger row
    /// ([`record_drag`]); `fp_before` is the manual store's fingerprint taken
    /// **before** the commit. Also drops the height redo tail: a new operation.
    /// Returns the row's `seq`, `None` (and records nothing) for an impossible range.
    pub(crate) fn record_way_drag(&mut self, first: usize, count: usize, fp_before: u64) -> Option<u64> {
        let fps: Vec<(WayStore, u64)> = WayStore::ALL.iter().map(|&s| (s, self.way_fp(s))).collect();
        let fp = |s: WayStore| fps.iter().find(|(k, _)| *k == s).map_or(0, |(_, v)| *v);
        let manual = self.infra.as_ref().map(|t| t.ways.as_slice()).unwrap_or(&[]);
        let seq = record_drag(&mut self.ledger, &mut self.way_undo, manual, first, count, fp_before, &fp)?;
        self.redo.clear();
        Some(seq)
    }
}

/// `"road “Old Mill Road”"`, or `"unnamed track"` -- a way described, never
/// given a name it does not have.
fn held_way_title(w: &HeldWay) -> String {
    let (kind, name) = match w {
        HeldWay::Generated(w) => (generated_way_type_key(w.way_type), w.name.as_str()),
        HeldWay::SeaLane(w) => ("sea_lane", w.name.as_str()),
        HeldWay::Manual(w) => (crate::infra_tools_bridge::way_type_key(w.way_type), w.name.as_str()),
        // A drag is titled by its first way's type and is never given a name (the
        // drag path commits unnamed ways). Real labels come from `drag_label`.
        HeldWay::Drag(ws) => (ws.first().map_or("way", |w| crate::infra_tools_bridge::way_type_key(w.way_type)), ""),
    };
    let kind = kind.replace('_', " ");
    if name.is_empty() { format!("unnamed {kind}") } else { format!("{kind} “{name}”") }
}

/// A grid-space point as the `Vector2` Godot receives (narrowed to `f32`;
/// the display layer does not need `f64`).
fn to_v2(p: (f64, f64)) -> Vector2 {
    Vector2::new(p.0 as f32, p.1 as f32)
}

/// The Godot-facing picks and the one write (`way_delete`); the second
/// `#[godot_api(secondary)]` block, like `campaign_bridge.rs`.
#[godot_api(secondary)]
impl WorldGen {
    /// The way under `(gx, gy)` (grid coordinates), within `radius_cells` of
    /// its **drawn** curve -- the same `way_render_polyline` resample
    /// `get_roads()`/`get_sea_routes()` hand the overlay, breaks honoured.
    /// Hidden (consolidated-away) ways are not drawn and are not hit.
    ///
    /// `{store, index, way_type, km, manual, x, y, dist}` plus `name` --
    /// **omitted** when the way has none. `store`/`index` are what
    /// [`Self::way_get`] and [`Self::way_delete`] take. Empty when nothing is
    /// in range. Read-only.
    #[func]
    fn way_pick(&self, gx: f64, gy: f64, radius_cells: f64) -> VarDictionary {
        let mut geo: Vec<(WayStore, usize, Vec<(f64, f64)>, Vec<usize>)> = Vec::new();
        if let Some(civ) = self.civ.as_ref() {
            for (i, w) in civ.ways.iter().enumerate().filter(|(_, w)| !w.hidden) {
                let (p, b) = crate::way_render_polyline(&w.pts, &w.brks);
                geo.push((WayStore::Generated, i, p, b));
            }
            for (i, w) in civ.sea_routes.iter().enumerate() {
                let (p, b) = crate::way_render_polyline(&w.pts, &w.brks);
                geo.push((WayStore::SeaLane, i, p, b));
            }
        }
        if let Some(t) = self.infra.as_ref() {
            for (i, w) in t.ways.iter().enumerate().filter(|(_, w)| !w.hidden) {
                let (p, b) = crate::way_render_polyline(&w.pts, &w.brks);
                geo.push((WayStore::Manual, i, p, b));
            }
        }
        let cands: Vec<(&[(f64, f64)], &[usize])> = geo.iter().map(|g| (g.2.as_slice(), g.3.as_slice())).collect();
        let Some((ci, dist, at)) = nearest_polyline(&cands, (gx, gy), radius_cells) else {
            return VarDictionary::new();
        };
        let (store, index) = (geo[ci].0, geo[ci].1);
        let mut d = self.way_get(GString::from(store.key()), index as i64);
        d.remove("points");
        d.remove("brks");
        d.set("x", at.0);
        d.set("y", at.1);
        d.set("dist", dist);
        d
    }

    /// One way, whole: `{store, index, points (raw PackedVector2Array),
    /// brks, km, way_type, manual, hidden}`, plus `name` when it has one and
    /// `tid` for a generated way. `points` are the stored control points
    /// (`Way::pts`), not the drawn resample -- what a delete removes and an
    /// undo must put back. Empty for an unknown store or index.
    #[func]
    fn way_get(&self, store: GString, index: i64) -> VarDictionary {
        let Some(store) = WayStore::parse(&store.to_string()) else { return VarDictionary::new() };
        let Ok(i) = usize::try_from(index) else { return VarDictionary::new() };
        let (pts, brks, km, name, way_type, hidden, tid): (&[(f64, f64)], &[usize], f64, &str, &str, bool, Option<u64>) = match store {
            WayStore::Generated => {
                let Some(w) = self.civ.as_ref().and_then(|c| c.ways.get(i)) else { return VarDictionary::new() };
                (&w.pts, &w.brks, w.km, &w.name, generated_way_type_key(w.way_type), w.hidden, Some(w.tid))
            }
            WayStore::SeaLane => {
                let Some(w) = self.civ.as_ref().and_then(|c| c.sea_routes.get(i)) else { return VarDictionary::new() };
                (&w.pts, &w.brks, w.km, &w.name, "sea_lane", false, None)
            }
            WayStore::Manual => {
                let Some(w) = self.infra.as_ref().and_then(|t| t.ways.get(i)) else { return VarDictionary::new() };
                (&w.pts, &w.brks, w.km, &w.name, crate::infra_tools_bridge::way_type_key(w.way_type), w.hidden, None)
            }
        };
        let points: PackedVector2Array = pts.iter().map(|&p| to_v2(p)).collect();
        let brks: PackedInt32Array = brks.iter().map(|&b| b as i32).collect();
        let mut d = vdict! {
            "store" => store.key(),
            "index" => i as i64,
            "points" => &points,
            "brks" => &brks,
            "km" => km,
            "way_type" => way_type,
            "manual" => store == WayStore::Manual,
            "hidden" => hidden,
        };
        if !name.is_empty() {
            d.set("name", name);
        }
        if let Some(t) = tid {
            d.set("tid", t as i64);
        }
        d
    }

    /// Delete one way (Ruling BA: **undoable**). Records a
    /// [`undo::EntryKind::WayDelete`] ledger row and holds the way, so
    /// `undo_last()` puts it back at the same index. A new operation, so the
    /// redo tail is dropped, as `carve_fjords` and `sculpt_commit` drop it.
    ///
    /// Indices after `index` in the same store shift down by one, the
    /// reference's own `splice` -- a caller holding one re-reads. `false` for
    /// an unknown store or an out-of-range index; nothing is recorded then.
    #[func]
    fn way_delete(&mut self, store: GString, index: i64) -> bool {
        let Some(store) = WayStore::parse(&store.to_string()) else { return false };
        let Ok(i) = usize::try_from(index) else { return false };
        let fp_before = self.way_fp(store);
        let held = {
            let (civ, infra) = (self.civ.as_mut(), self.infra.as_mut());
            let (generated, sea) = match civ {
                Some(c) => (Some(&mut c.ways), Some(&mut c.sea_routes)),
                None => (None, None),
            };
            let mut st = WayStores { generated, sea, manual: infra.map(|t| &mut t.ways) };
            take_way(&mut st, store, i)
        };
        let Some(way) = held else { return false };
        let fp_after = self.way_fp(store);
        self.redo.clear();
        let fps: Vec<(WayStore, u64)> = WayStore::ALL.iter().map(|&s| (s, self.way_fp(s))).collect();
        let fp = |s: WayStore| fps.iter().find(|(k, _)| *k == s).map_or(0, |(_, v)| *v);
        // The ledger row, the redo drop and the prune (against this store's
        // PRE-delete fingerprint, R1) live in the pure `record_delete`.
        record_delete(&mut self.ledger, &mut self.way_undo, store, i, way, fp_before, fp_after, &fp);
        true
    }

    /// The river under `(gx, gy)`, resolved as Ruling BA's "branch to its
    /// mouth": the nearest channel cell (`stream_order >= 1`) within
    /// `radius_cells`, and the walk downstream from it (see
    /// [`trace_branch`]).
    ///
    /// `{cell, x, y, order, dist, branch (PackedVector2Array of cell
    /// centres, clicked cell first), cells, km, terminus}` -- `terminus` is
    /// `"sea"`, `"lake"`, `"inland"` or `"edge"`. No `name`: rivers are unnamed
    /// in this engine. `x`/`y` are the clicked cell's centre, `order` its own
    /// Strahler order. Empty for a world with no traced network (a loaded
    /// save) or nothing in range.
    #[func]
    fn river_pick(&self, gx: f64, gy: f64, radius_cells: f64) -> VarDictionary {
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else { return VarDictionary::new() };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let (Some(order), Some(ch)) = (ws.stream_order.as_deref(), ws.channels.as_ref()) else {
            return VarDictionary::new();
        };
        let Some((cell, dist)) = nearest_channel_cell(order, gw, gh, self.world, gx, gy, radius_cells) else {
            return VarDictionary::new();
        };
        // The water the map DRAWS (`drawn_water_classification`, forced
        // lakes included), so the card's "ends in a lake" names a lake that is
        // on screen. It read `CivData::water_bodies` first until 2026-09-28 --
        // a copy taken at generation and never refreshed by a sculpt, so the
        // card could say "lake" where the map showed land (`OUTSTANDING_WORK.md`
        // "Four map-data defects", item 1). No world, no classification: the
        // walk then treats every cell as land, as it did for a missing grid.
        let drawn = self.drawn_water_classification();
        let classes: &[u8] = drawn.as_deref().unwrap_or(&[]);
        let water = |i: usize| classes.get(i).copied().unwrap_or(0);
        // The accumulation tree, built only if the channel chain stops on
        // land -- one whole-grid pass, so not paid on a chain that reaches the
        // sea by itself.
        let mut flow_recv: Option<Vec<i32>> = None;
        let mut cont = |i: usize| -> i64 {
            let r = flow_recv.get_or_insert_with(|| {
                let route = cartalith_hydrology::routing_view(&ws.field, gw, gh, self.sea_level, self.world, ws.integrated_drainage);
                cartalith_hydrology::flow_receivers(&route, gw, gh, self.world)
            });
            r.get(i).copied().unwrap_or(-1) as i64
        };
        let (cells, terminus) = trace_branch(cell, gw, gh, self.world, &ch.recv, &mut cont, &water);
        let ck = cartalith_spatial::cell_km(self.map_width_km, gw);
        let branch: PackedVector2Array =
            cells.iter().map(|&c| Vector2::new((c % gw) as f32 + 0.5, (c / gw) as f32 + 0.5)).collect();
        vdict! {
            "cell" => cell as i64,
            "x" => (cell % gw) as f64 + 0.5,
            "y" => (cell / gw) as f64 + 0.5,
            "order" => order[cell] as i64,
            "dist" => dist,
            "branch" => &branch,
            "cells" => cells.len() as i64,
            "km" => branch_cells_length(&cells, gw, self.world) * ck,
            "terminus" => terminus.key(),
        }
    }

    /// Every cell draining into `cell`, `cell` included, as a `gw * gh` mask
    /// (`1` = drains here). "Draining" follows **the same tree the branch
    /// walks down** ([`combined_receivers`]): a channel cell's channel
    /// receiver, every other cell's flow-accumulation receiver
    /// (`flow_receivers` over the routing surface). So a cell is in the
    /// catchment exactly when *Trace downstream* from it would pass through
    /// `cell`.
    ///
    /// Not the accumulation tree alone, and that is measured, not preferred:
    /// the channel network is a D-infinity aspect projection and the
    /// accumulation tree is plain D8 (`cartalith_hydrology::River::discharge`'s
    /// doc), so a channel cell can sit off the D8 main path. Over the first 80
    /// order >= 2 cells of three 192x144 worlds (seeds 20260902, 719004, 5521;
    /// a scratch test, not kept) the D8 tree alone gave a catchment of three
    /// cells or fewer for 7, 31 and 1 of them; this combined tree for none.
    /// Empty for a loaded save or an out-of-range cell.
    #[func]
    fn river_catchment(&self, cell: i64) -> PackedByteArray {
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else { return PackedByteArray::new() };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let Ok(c) = usize::try_from(cell) else { return PackedByteArray::new() };
        if c >= gw * gh || ws.field.len() != gw * gh {
            return PackedByteArray::new();
        }
        let Some(ch) = ws.channels.as_ref() else { return PackedByteArray::new() };
        let route = cartalith_hydrology::routing_view(&ws.field, gw, gh, self.sea_level, self.world, ws.integrated_drainage);
        let d8 = cartalith_hydrology::flow_receivers(&route, gw, gh, self.world);
        PackedByteArray::from(catchment_mask(&combined_receivers(&ch.recv, &d8), c).as_slice())
    }

    /// A `w * h` 0/1 mask (`river_catchment`'s) as an RGBA8 image: `color`
    /// where the mask is set, fully transparent elsewhere -- what the overlay
    /// draws over the map. `None` when the mask is not `w * h` long. Done here
    /// because the per-pixel expansion is millions of iterations at the
    /// shipping grid size, which GDScript would spend seconds on.
    #[func]
    fn mask_rgba_image(&self, mask: PackedByteArray, w: i64, h: i64, color: Color) -> Option<Gd<godot::classes::Image>> {
        let (Ok(wu), Ok(hu)) = (usize::try_from(w), usize::try_from(h)) else { return None };
        let m = mask.as_slice();
        if wu == 0 || hu == 0 || m.len() != wu * hu {
            return None;
        }
        let q = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        let px = [q(color.r), q(color.g), q(color.b), q(color.a)];
        let mut bytes = vec![0u8; m.len() * 4];
        for (i, &v) in m.iter().enumerate() {
            if v != 0 {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&px);
            }
        }
        godot::classes::Image::create_from_data(w as i32, h as i32, false, godot::classes::image::Format::RGBA8, &PackedByteArray::from(bytes.as_slice()))
    }

    /// What `undo_last()` would revert next: `"height"`, `"civ"` (a way
    /// delete) or `""`. The shell asks before undoing so it repaints the right
    /// layer -- a way comes back on the overlay, not in the height texture.
    #[func]
    fn undo_next_subsystem(&self) -> GString {
        GString::from(match self.next_undo_step() {
            Some(UndoPick::Height(_)) => "height",
            Some(UndoPick::Way { .. }) => "civ",
            None => "",
        })
    }
}

/// Tests for the pure halves of CM-7's rows: the way pick, the held-way undo
/// and its fingerprint liveness, and the river branch and catchment walks.
/// The `#[func]` wrappers need a live `WorldGen` and are covered by probe scenes.
#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::tools::ManualWayType;
    use cartalith_civ::WayType;

    /// A generated way with `km` 12.5 (judgement: any value, the tests only need
    /// it stable) and the name `W<tid>`.
    fn way(tid: u64, pts: Vec<(f64, f64)>, t: WayType) -> Way {
        Way { tid, pts, brks: vec![], km: 12.5, name: format!("W{tid}"), way_type: t, a_idx: 0, b_idx: 1, hidden: false }
    }

    /// A hand-drawn land track with `km` 3.0 (judgement) and no name.
    fn manual(pts: Vec<(f64, f64)>) -> ManualWay {
        ManualWay { pts, brks: vec![], km: 3.0, sea: false, way_type: ManualWayType::Track, name: String::new(), hidden: false }
    }

    // -- pick ------------------------------------------------------------

    /// Protects: `nearest_polyline` choosing the closer way, reporting the foot
    /// point, and returning nothing out of tolerance. A horizontal and a vertical way crossing near (10, 10); the pointer at
    /// (10.4, 13) is 0.4 cells from the vertical one and 3 from the other.
    #[test]
    fn nearest_polyline_picks_the_closer_way_and_reports_the_foot() {
        let h = [(0.0, 10.0), (20.0, 10.0)];
        let v = [(10.0, 0.0), (10.0, 20.0)];
        let cands: Vec<(&[(f64, f64)], &[usize])> = vec![(&h, &[]), (&v, &[])];
        let (ci, d, at) = nearest_polyline(&cands, (10.4, 13.0), 1.0).unwrap();
        assert_eq!(ci, 1);
        assert!((d - 0.4).abs() < 1e-12, "{d}");
        assert_eq!(at, (10.0, 13.0));
        // Out of tolerance: nothing, not the nearest-anyway.
        assert_eq!(nearest_polyline(&cands, (13.0, 14.0), 1.0), None);
    }

    /// Protects: breaks as gaps. A break at index 2 means the stroke `pts[1]`-`pts[2]` is not drawn, so
    /// a pointer on the gap misses; the same pointer hits without the break.
    #[test]
    fn a_break_is_a_gap_the_pick_does_not_hit() {
        let pts = [(0.0, 0.0), (4.0, 0.0), (10.0, 0.0), (14.0, 0.0)];
        let with: Vec<(&[(f64, f64)], &[usize])> = vec![(&pts, &[2])];
        let without: Vec<(&[(f64, f64)], &[usize])> = vec![(&pts, &[])];
        assert_eq!(nearest_polyline(&with, (7.0, 0.5), 1.0), None);
        let (ci, d, _) = nearest_polyline(&without, (7.0, 0.5), 1.0).unwrap();
        assert_eq!(ci, 0);
        assert!((d - 0.5).abs() < 1e-12);
        // Either run on its own still hits.
        assert_eq!(nearest_polyline(&with, (2.0, 0.25), 1.0).map(|h| h.0), Some(0));
        assert_eq!(nearest_polyline(&with, (12.0, 0.25), 1.0).map(|h| h.0), Some(0));
    }

    /// Protects: a single-point polyline is hit at that point, with the distance to it.
    #[test]
    fn a_one_point_run_is_hit_at_its_point() {
        let p = [(5.0, 5.0)];
        let c: Vec<(&[(f64, f64)], &[usize])> = vec![(&p, &[])];
        let (_, d, at) = nearest_polyline(&c, (5.0, 5.6), 1.0).unwrap();
        assert!((d - 0.6).abs() < 1e-12);
        assert_eq!(at, (5.0, 5.0));
    }

    // -- delete / undo -----------------------------------------------------

    /// A fingerprint lookup over the three given stores, standing in for
    /// `WorldGen::way_fp`.
    fn fps(g: &[Way], s: &[SeaRoute], m: &[ManualWay]) -> impl Fn(WayStore) -> u64 {
        let (a, b, c) = (fp_generated(Some(g)), fp_sea(Some(s)), fp_manual(Some(m)));
        move |st| match st {
            WayStore::Generated => a,
            WayStore::SeaLane => b,
            WayStore::Manual => c,
        }
    }

    /// Protects: undo fidelity. Delete the middle generated way, then put it back: the list is
    /// byte-identical to before -- endpoints, points, type, tid, index.
    #[test]
    fn take_then_put_restores_the_way_exactly_at_its_index() {
        let mut g = vec![
            way(1, vec![(0.0, 0.0), (5.0, 5.0)], WayType::Road),
            way(2, vec![(3.0, 1.0), (6.0, 2.0), (9.0, 4.0)], WayType::Highway),
            way(3, vec![(7.0, 7.0), (8.0, 9.0)], WayType::Track),
        ];
        let before = g.clone();
        let mut m = vec![manual(vec![(1.0, 1.0), (2.0, 2.0)])];
        let held = {
            let mut st = WayStores { generated: Some(&mut g), sea: None, manual: Some(&mut m) };
            take_way(&mut st, WayStore::Generated, 1).unwrap()
        };
        assert_eq!(g.len(), 2);
        assert_eq!(g[1].tid, 3, "the later way shifted down");
        match &held {
            HeldWay::Generated(w) => {
                assert_eq!(w.tid, 2);
                assert_eq!(w.pts, vec![(3.0, 1.0), (6.0, 2.0), (9.0, 4.0)]);
                assert_eq!(w.way_type, WayType::Highway);
            }
            other => panic!("wrong store: {other:?}"),
        }
        let mut st = WayStores { generated: Some(&mut g), sea: None, manual: Some(&mut m) };
        assert!(put_way(&mut st, 1, held));
        assert_eq!(g, before);
    }

    /// Protects: `take_way` and `put_way` refuse a missing store or an out-of-range
    /// index and write nothing.
    #[test]
    fn take_refuses_a_missing_store_or_index() {
        let mut g = vec![way(1, vec![(0.0, 0.0), (1.0, 0.0)], WayType::Road)];
        let mut st = WayStores { generated: Some(&mut g), sea: None, manual: None };
        assert_eq!(take_way(&mut st, WayStore::Generated, 1), None);
        assert_eq!(take_way(&mut st, WayStore::Manual, 0), None);
        assert_eq!(take_way(&mut st, WayStore::SeaLane, 0), None);
        assert!(!put_way(&mut st, 5, HeldWay::Generated(way(9, vec![], WayType::Road))), "past the end");
        assert_eq!(g.len(), 1);
    }

    /// Protects: the fingerprint chain deciding liveness. Two deletes from one store are both live and undo newest first; after
    /// the store is rebuilt under them, neither is.
    #[test]
    fn the_fingerprint_chain_decides_liveness() {
        let mut g = vec![
            way(1, vec![(0.0, 0.0), (5.0, 0.0)], WayType::Road),
            way(2, vec![(0.0, 2.0), (5.0, 2.0)], WayType::Road),
            way(3, vec![(0.0, 4.0), (5.0, 4.0)], WayType::Road),
        ];
        let (s, m): (Vec<SeaRoute>, Vec<ManualWay>) = (vec![], vec![]);
        let mut u = WayUndo::new();
        for (seq, idx) in [(10u64, 0usize), (11, 1)] {
            let fb = fp_generated(Some(&g));
            let w = HeldWay::Generated(g.remove(idx));
            u.push(WayUndoEntry { seq, index: idx, way: w, fp_before: fb, fp_after: fp_generated(Some(&g)) });
        }
        assert_eq!(g.len(), 1);
        assert_eq!(u.live_seqs(&fps(&g, &s, &m)), vec![11, 10]);
        assert_eq!(u.newest_live(&fps(&g, &s, &m)).map(|e| e.seq), Some(11));

        // Something else edits the list: the chain is broken at the top.
        let mut rebuilt = g.clone();
        rebuilt.push(way(9, vec![(1.0, 1.0), (2.0, 1.0)], WayType::Track));
        assert_eq!(u.live_seqs(&fps(&rebuilt, &s, &m)), Vec::<u64>::new());
        let mut pruned = WayUndo::new();
        for e in &u.entries {
            pruned.push(e.clone());
        }
        pruned.prune_stale(&fps(&rebuilt, &s, &m));
        assert_eq!(pruned.len(), 0);

        // A delete made after an unrelated edit is live; the older one below
        // the edit is not.
        let mut v = WayUndo::new();
        let mut g2 = vec![
            way(1, vec![(0.0, 0.0), (5.0, 0.0)], WayType::Road),
            way(2, vec![(0.0, 2.0), (5.0, 2.0)], WayType::Road),
        ];
        let fb = fp_generated(Some(&g2));
        let w = HeldWay::Generated(g2.remove(0));
        v.push(WayUndoEntry { seq: 20, index: 0, way: w, fp_before: fb, fp_after: fp_generated(Some(&g2)) });
        g2[0].name = "Renamed".into();
        let fb = fp_generated(Some(&g2));
        let w = HeldWay::Generated(g2.remove(0));
        v.push(WayUndoEntry { seq: 21, index: 0, way: w, fp_before: fb, fp_after: fp_generated(Some(&g2)) });
        assert_eq!(v.live_seqs(&fps(&g2, &s, &m)), vec![21]);
    }

    /// Protects: per-store chains. A manual delete stays live when the generated
    /// network is rebuilt.
    #[test]
    fn a_rebuild_of_one_store_leaves_the_others_live() {
        let g = vec![way(1, vec![(0.0, 0.0), (5.0, 0.0)], WayType::Road)];
        let mut m = vec![manual(vec![(1.0, 1.0), (3.0, 3.0)]), manual(vec![(4.0, 4.0), (6.0, 6.0)])];
        let s: Vec<SeaRoute> = vec![];
        let mut u = WayUndo::new();
        let fb = fp_manual(Some(&m));
        let w = HeldWay::Manual(m.remove(1));
        u.push(WayUndoEntry { seq: 5, index: 1, way: w, fp_before: fb, fp_after: fp_manual(Some(&m)) });
        let g_rebuilt = vec![way(7, vec![(2.0, 0.0), (9.0, 0.0)], WayType::Highway)];
        assert_eq!(u.live_seqs(&fps(&g, &s, &m)), vec![5]);
        assert_eq!(u.live_seqs(&fps(&g_rebuilt, &s, &m)), vec![5]);
    }

    /// Protects: the fingerprint noticing an interior change. Same ends, same length, same name -- a re-routed interior is still a
    /// different list, and a held way must not be re-inserted into it.
    #[test]
    fn the_fingerprint_sees_a_rerouted_interior() {
        let a = vec![way(1, vec![(0.0, 0.0), (5.0, 5.0)], WayType::Road)];
        let b = vec![way(1, vec![(0.0, 0.0), (2.0, 4.0), (5.0, 5.0)], WayType::Road)];
        assert_ne!(fp_generated(Some(&a)), fp_generated(Some(&b)));
        assert_ne!(fp_generated(Some(&a)), fp_generated(None), "no store is not an empty store");
        assert_eq!(fp_generated(Some(&a)), fp_generated(Some(&a.clone())));
    }

    /// Protects: `choose_undo` picks the higher `seq` of the height and way rows,
    /// and a lone candidate wins.
    #[test]
    fn the_newer_of_height_and_way_is_undone_first() {
        let h = || Some((5u64, "Sculpt commit".to_string()));
        let w = |s: u64| Some((s, "Delete road “A”".to_string()));
        assert_eq!(choose_undo(h(), w(6)), Some(UndoPick::Way { seq: 6, label: "Delete road “A”".into() }));
        assert_eq!(choose_undo(h(), w(4)), Some(UndoPick::Height("Sculpt commit".into())));
        assert_eq!(choose_undo(None, w(4)), Some(UndoPick::Way { seq: 4, label: "Delete road “A”".into() }));
        assert_eq!(choose_undo(h(), None), Some(UndoPick::Height("Sculpt commit".into())));
        assert_eq!(choose_undo(None, None), None);
    }

    /// Protects: `held_way_title` describes an unnamed way as `unnamed <type>` and
    /// quotes a real name, never inventing one.
    #[test]
    fn a_held_way_is_described_not_named() {
        assert_eq!(held_way_title(&HeldWay::Manual(manual(vec![]))), "unnamed track");
        assert_eq!(held_way_title(&HeldWay::Generated(way(4, vec![], WayType::Highway))), "highway “W4”");
    }

    // -- river -------------------------------------------------------------

    /// 6x4, no wrap. A channel runs along row 1 from x=1 to x=4; x=5 row 1
    /// is sea.
    /// The shared river grid (see the comment below). Returns
    /// `(gw, gh, order, recv, water)`.
    fn river_fixture() -> (usize, usize, Vec<i16>, Vec<i32>, Vec<u8>) {
        let (gw, gh) = (6usize, 4usize);
        let n = gw * gh;
        let mut order = vec![0i16; n];
        let mut recv = vec![-1i32; n];
        let mut water = vec![0u8; n];
        for x in 1..=4 {
            order[gw + x] = if x < 3 { 1 } else { 2 };
            recv[gw + x] = (gw + x + 1) as i32;
        }
        water[gw + 5] = 1;
        (gw, gh, order, recv, water)
    }

    /// Protects: `nearest_channel_cell` picks the closest channel centre inside the
    /// radius and returns nothing beyond it.
    #[test]
    fn nearest_channel_cell_is_the_closest_centre_in_range() {
        let (gw, gh, order, _, _) = river_fixture();
        // (2.2, 1.9): cell 8 (x=2,y=1) centre (2.5,1.5) is 0.5 away.
        let (c, d) = nearest_channel_cell(&order, gw, gh, false, 2.2, 1.9, 0.75).unwrap();
        assert_eq!(c, 8);
        assert!((d - 0.5).abs() < 1e-12, "{d}");
        // Two rows below the channel: out of range at 0.75, in range at 2.
        assert_eq!(nearest_channel_cell(&order, gw, gh, false, 2.5, 3.5, 0.75), None);
        assert_eq!(nearest_channel_cell(&order, gw, gh, false, 2.5, 3.5, 2.0).map(|h| h.0), Some(8));
    }

    /// Protects: the channel walk to the sea. From x=2 the chain runs 8 -> 9 -> 10 -> 11 (sea), 3 steps: the branch
    /// stops on the first sea cell and never asks the fallback tree.
    #[test]
    fn the_branch_runs_to_the_sea_along_the_channel() {
        let (gw, gh, _, recv, water) = river_fixture();
        let mut asked = 0;
        let mut cont = |_: usize| -> i64 {
            asked += 1;
            -1
        };
        let (cells, t) = trace_branch(8, gw, gh, false, &recv, &mut cont, &|i| water[i]);
        assert_eq!(cells, vec![8, 9, 10, 11]);
        assert_eq!(t, Terminus::Sea);
        assert_eq!(asked, 0);
        assert_eq!(branch_cells_length(&cells, gw, false), 3.0);
    }

    /// Protects: the fallback to the flow tree. Cut the channel at x=3 (cell 9 has no channel receiver): the walk
    /// continues on the fallback tree, 9 -> 16 (x=4,y=2) -> 17 (x=5,y=2),
    /// which is a lake.
    #[test]
    fn a_channel_that_stops_on_land_continues_on_the_flow_tree() {
        let (gw, gh, _, mut recv, mut water) = river_fixture();
        recv[9] = -1;
        water[17] = 2;
        let mut cont = |i: usize| -> i64 {
            match i {
                9 => 16,
                16 => 17,
                _ => -1,
            }
        };
        let (cells, t) = trace_branch(7, gw, gh, false, &recv, &mut cont, &|i| water[i]);
        assert_eq!(cells, vec![7, 8, 9, 16, 17]);
        assert_eq!(t, Terminus::Lake);
        let len = branch_cells_length(&cells, gw, false);
        assert!((len - (3.0 + 2f64.sqrt())).abs() < 1e-12, "{len}");
    }

    /// Protects: a walk that stops on land is `Inland` or `MapEdge` (edge only on a
    /// non-wrapping border), never `Sea`.
    #[test]
    fn a_dead_end_is_inland_or_the_map_edge_never_the_sea() {
        let (gw, gh, _, mut recv, water) = river_fixture();
        recv[9] = -1;
        let (cells, t) = trace_branch(8, gw, gh, false, &recv, &mut |_| -1, &|i| water[i]);
        assert_eq!(cells, vec![8, 9]);
        assert_eq!(t, Terminus::Inland);
        // Cell 6 is x=0 on row 1: the western edge.
        let (_, t2) = trace_branch(6, gw, gh, false, &recv, &mut |_| -1, &|i| water[i]);
        assert_eq!(t2, Terminus::MapEdge);
        // ...which is not an edge on a wrapping world.
        let (_, t3) = trace_branch(6, gw, gh, true, &recv, &mut |_| -1, &|i| water[i]);
        assert_eq!(t3, Terminus::Inland);
    }

    /// Protects: `branch_cells_length` counts a seam crossing as one cell only when
    /// the world wraps.
    #[test]
    fn a_seam_step_is_one_cell_on_a_wrapping_world() {
        // 6 wide: cell 5 (x=5) to cell 0 (x=0) on row 0.
        assert_eq!(branch_cells_length(&[5, 0], 6, true), 1.0);
        assert_eq!(branch_cells_length(&[5, 0], 6, false), 5.0);
    }

    /// Protects: `combined_receivers` and its catchment. The channel receiver wins where there is one; the D8 one fills the rest.
    #[test]
    fn combined_receivers_prefers_the_channel_tree() {
        assert_eq!(combined_receivers(&[-1, 5, -1, 7], &[3, 2, -1, 1]), vec![3, 5, -1, 7]);
        // A hillslope cell (0) whose D8 step lands on a channel cell (3) that
        // the channel tree routes to 7: in 7's catchment through the combined
        // tree, not through the D8 tree alone (3 -> 1 there).
        let d8 = [3, -1, -1, 1, -1, -1, -1, -1];
        let comb = combined_receivers(&[-1, -1, -1, 7, -1, -1, -1, -1], &d8);
        assert_eq!(catchment_mask(&comb, 7), vec![1, 0, 0, 1, 0, 0, 0, 1]);
        assert_eq!(catchment_mask(&d8, 7), vec![0, 0, 0, 0, 0, 0, 0, 1]);
    }

    /// Protects: `catchment_mask` membership. A 3x3 tree: every cell drains to the centre (4) except 0, which drains
    /// to 1; 2 drains nowhere. The centre's catchment is everything but 2.
    #[test]
    fn the_catchment_is_every_cell_whose_chain_reaches_the_target() {
        let recv = vec![1, 4, -1, 4, -1, 4, 4, 4, 4];
        let m = catchment_mask(&recv, 4);
        assert_eq!(m, vec![1, 1, 0, 1, 1, 1, 1, 1, 1]);
        assert_eq!(catchment_mask(&recv, 1), vec![1, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(catchment_mask(&recv, 2), vec![0, 0, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(catchment_mask(&recv, 99), vec![0; 9]);
    }
}

/// Tests for ET-D4's `WayDrag` ledger rows: the pure record / undo / redo cores
/// ([`record_drag`], [`undo_way_core`], [`drag_redo_ready`], [`redo_drag_core`])
/// driven through a small harness that wires them exactly as `WorldGen`'s thin
/// wrappers do (`WorldGen` itself is a cdylib `GodotClass` and cannot be built in a
/// unit test). Each test names what it protects.
#[cfg(test)]
mod drag_ledger_tests {
    use super::*;
    use cartalith_civ::tools::ManualWayType;
    use undo::{EntryKind, HistoryLedger};

    /// A hand-drawn way of `t` with distinct, literal points (judgement: any
    /// stable points; `km` is derived from `x0` so two ways never compare equal).
    fn mw(t: ManualWayType, x0: f64) -> ManualWay {
        ManualWay {
            pts: vec![(x0, 1.0), (x0 + 1.0, 2.0), (x0 + 2.0, 2.5)],
            brks: vec![],
            km: 4.0 + x0,
            sea: t == ManualWayType::SeaLane,
            way_type: t,
            name: String::new(),
            hidden: false,
        }
    }

    /// The pieces `WorldGen` owns, with the same wiring as its wrappers.
    struct H {
        generated: Vec<Way>,
        sea: Vec<SeaRoute>,
        manual: Vec<ManualWay>,
        ledger: HistoryLedger,
        wu: WayUndo,
        /// Stand-in for `WorldGen::undo.depth()` (height snapshots held).
        depth: usize,
    }

    impl H {
        /// A world with an empty ledger holding one Floor row (seq 1), like a generate.
        fn new() -> H {
            let mut ledger = HistoryLedger::new();
            ledger.record("world", "Generate world", "", EntryKind::Floor);
            H { generated: vec![], sea: vec![], manual: vec![], ledger, wu: WayUndo::new(), depth: 0 }
        }

        /// `WorldGen::way_fp` over the harness stores.
        fn fp(&self) -> impl Fn(WayStore) -> u64 + use<> {
            let (a, b, c) = (fp_generated(Some(&self.generated)), fp_sea(Some(&self.sea)), fp_manual(Some(&self.manual)));
            move |s| match s {
                WayStore::Generated => a,
                WayStore::SeaLane => b,
                WayStore::Manual => c,
            }
        }

        /// What `way_stroke_end` does on success: push `ways`, then record the drag.
        fn commit_drag(&mut self, ways: Vec<ManualWay>) -> Option<u64> {
            let fp_before = fp_manual(Some(&self.manual));
            let (first, count) = (self.manual.len(), ways.len());
            self.manual.extend(ways);
            let fps = self.fp();
            record_drag(&mut self.ledger, &mut self.wu, &self.manual, first, count, fp_before, &fps)
        }

        /// A click-committed way: pushed with **no** ledger row, as `way_commit` does.
        fn click_commit(&mut self, w: ManualWay) {
            self.manual.push(w);
        }

        /// What `way_delete` does for a manual way.
        fn delete_manual(&mut self, i: usize) -> u64 {
            let fp_before = fp_manual(Some(&self.manual));
            let way = HeldWay::Manual(self.manual.remove(i));
            let fp_after = fp_manual(Some(&self.manual));
            let seq = self.ledger.record("civ", format!("Delete {}", held_way_title(&way)), "hand-drawn", EntryKind::WayDelete);
            self.wu.clear_redo();
            self.wu.push(WayUndoEntry { seq, index: i, way, fp_before, fp_after });
            seq
        }

        /// What `way_delete` does for a manual way, through the real
        /// [`record_delete`] (the older `delete_manual` above records the row
        /// by hand and never prunes, which is why it could not see R1).
        fn delete_real(&mut self, i: usize) -> u64 {
            let fp_before = fp_manual(Some(&self.manual));
            let way = HeldWay::Manual(self.manual.remove(i));
            let fp_after = fp_manual(Some(&self.manual));
            let fps = self.fp();
            record_delete(&mut self.ledger, &mut self.wu, WayStore::Manual, i, way, fp_before, fp_after, &fps)
        }

        /// `WorldGen::undo_way_step` over the harness.
        fn undo(&mut self) -> Option<WayStep> {
            let fps = self.fp();
            let mut st = WayStores { generated: Some(&mut self.generated), sea: Some(&mut self.sea), manual: Some(&mut self.manual) };
            undo_way_core(&mut self.ledger, &mut self.wu, &mut st, &fps, self.depth)
        }

        /// `redo_last()`'s drag branch: the label, or `None` when no drag is offered.
        fn redo(&mut self) -> Option<String> {
            redo_drag_core(&mut self.ledger, &mut self.wu, Some(&mut self.manual), self.depth)
        }

        /// Whether a redo is offered right now (`WorldGen::drag_redo_label`).
        fn redo_offered(&self) -> Option<String> {
            drag_redo_ready(&self.wu, &self.ledger, Some(&self.manual), self.depth).map(|r| r.label.clone())
        }

        /// The ledger's labels, oldest first.
        fn labels(&self) -> Vec<String> {
            self.ledger.rows(0).iter().map(|(e, _)| e.label.clone()).collect()
        }

        /// Whether the row `seq` is currently offered as reversible (`undo_ledger`'s `reversible`).
        fn reversible(&self, seq: u64) -> bool {
            let live = self.wu.live_seqs(&self.fp());
            self.ledger.rows_with(self.depth, &|s| live.contains(&s)).iter().any(|(e, l)| e.seq == seq && *l)
        }
    }

    /// Protects: a committed drag is exactly one `WayDrag` ledger row labelled `Draw road (2 ways)` for a mirrored pair, undo removes exactly the drag's ways (a pre-existing way is untouched) and the row, redo re-inserts them value-equal at the same index with a NEW row, and the list equals the pre-undo list.
    #[test]
    fn commit_undo_redo_round_trips_the_ways_byte_equal() {
        let mut h = H::new();
        h.click_commit(mw(ManualWayType::Track, 50.0));
        let (a, b) = (mw(ManualWayType::Road, 10.0), mw(ManualWayType::Road, 20.0));
        let seq = h.commit_drag(vec![a.clone(), b.clone()]).expect("a real drag records");
        assert_eq!(h.labels(), vec!["Generate world", "Draw road (2 ways)"]);
        assert!(h.reversible(seq));
        let after_commit = h.manual.clone();
        assert_eq!(after_commit.len(), 3);

        let step = h.undo().expect("a live drag undoes");
        assert_eq!(step, WayStep { label: "Draw road (2 ways)".into(), was_drag: true });
        assert_eq!(h.manual, vec![mw(ManualWayType::Track, 50.0)], "only the pre-existing way remains");
        assert_eq!(h.labels(), vec!["Generate world"], "the undone row left the ledger");
        assert_eq!(h.redo_offered().as_deref(), Some("Draw road (2 ways)"));

        assert_eq!(h.redo().as_deref(), Some("Draw road (2 ways)"));
        assert_eq!(h.manual, after_commit, "same ways, same indices, value-equal");
        assert_eq!(h.manual[1], a);
        assert_eq!(h.manual[2], b);
        assert_eq!(h.labels(), vec!["Generate world", "Draw road (2 ways)"]);
        let new_seq = h.ledger.last_seq();
        assert!(new_seq > seq, "the redone row is a new row: {new_seq} vs {seq}");
        assert!(h.reversible(new_seq), "and it is undoable again");
        assert_eq!(h.undo().map(|s| s.was_drag), Some(true));
        assert_eq!(h.manual.len(), 1);
    }

    /// Protects: labels follow the file's `Delete <title>` style with `Draw <type>`: road, track, sea lane (underscore turned to a space) and ancient each read literally, a mirrored pair carries its way count, and an empty list gets the neutral label.
    #[test]
    fn labels_name_the_way_type() {
        let mut h = H::new();
        for (t, want) in [
            (ManualWayType::Road, "Draw road"),
            (ManualWayType::Track, "Draw track"),
            (ManualWayType::SeaLane, "Draw sea lane"),
            (ManualWayType::Ancient, "Draw ancient"),
        ] {
            h.commit_drag(vec![mw(t, 1.0)]).unwrap();
            assert_eq!(h.labels().last().map(String::as_str), Some(want));
        }
        assert_eq!(drag_label(&[mw(ManualWayType::Road, 1.0), mw(ManualWayType::Road, 2.0)]), "Draw road (2 ways)");
        assert_eq!(drag_label(&[]), "Draw way");
    }

    /// Protects: a refused or empty drag adds no ledger row and no held entry: a zero count, a range past the store and an overflowing range all record nothing, leave `last_seq` and the rows unchanged, and hold nothing to undo.
    #[test]
    fn a_refused_or_empty_drag_records_nothing() {
        let mut h = H::new();
        h.click_commit(mw(ManualWayType::Road, 1.0));
        let (seq0, rows0) = (h.ledger.last_seq(), h.labels());
        let fps = h.fp();
        let fpb = fp_manual(Some(&h.manual));
        assert_eq!(record_drag(&mut h.ledger, &mut h.wu, &h.manual, 1, 0, fpb, &fps), None, "count 0");
        assert_eq!(record_drag(&mut h.ledger, &mut h.wu, &h.manual, 0, 2, fpb, &fps), None, "range past the end");
        assert_eq!(record_drag(&mut h.ledger, &mut h.wu, &h.manual, usize::MAX, 2, fpb, &fps), None, "overflowing range, no panic");
        assert_eq!((h.ledger.last_seq(), h.labels()), (seq0, rows0));
        assert!(h.undo().is_none(), "nothing was held");
    }

    /// Protects: undo twice and redo twice are safe: the second undo and the second redo of one drag do nothing and change nothing, with the ledger and list as after the first.
    #[test]
    fn repeated_undo_and_redo_are_safe() {
        let mut h = H::new();
        h.commit_drag(vec![mw(ManualWayType::Road, 3.0)]).unwrap();
        assert!(h.undo().is_some());
        let (rows, manual, seq) = (h.labels(), h.manual.clone(), h.ledger.last_seq());
        assert!(h.undo().is_none(), "second undo");
        assert_eq!((h.labels(), h.manual.clone(), h.ledger.last_seq()), (rows, manual, seq));
        assert!(h.redo().is_some());
        let (rows, manual, seq) = (h.labels(), h.manual.clone(), h.ledger.last_seq());
        assert!(h.redo().is_none(), "second redo");
        assert_eq!((h.labels(), h.manual.clone(), h.ledger.last_seq()), (rows, manual, seq));
    }

    /// Protects: two drags are a LIFO pair: undo, undo, redo, redo restores both in their original order at their original indices, every `seq` stays strictly increasing, and the redo order is the reverse of the undo order.
    #[test]
    fn two_drags_undo_and_redo_in_stack_order() {
        let mut h = H::new();
        let s1 = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        let s2 = h.commit_drag(vec![mw(ManualWayType::Track, 9.0)]).unwrap();
        assert!(s1 < s2);
        let both = h.manual.clone();
        assert_eq!(h.undo().unwrap().label, "Draw track", "newest first");
        assert_eq!(h.undo().unwrap().label, "Draw road");
        assert!(h.manual.is_empty());
        assert_eq!(h.wu.redo_len(), 2);
        assert_eq!(h.redo().as_deref(), Some("Draw road"), "the last undone is redone first");
        assert_eq!(h.manual, vec![both[0].clone()]);
        assert_eq!(h.redo().as_deref(), Some("Draw track"));
        assert_eq!(h.manual, both);
        let seqs: Vec<u64> = h.ledger.rows(0).iter().map(|(e, _)| e.seq).collect();
        assert!(seqs.windows(2).all(|w| w[0] < w[1]), "monotone: {seqs:?}");
        assert!(h.ledger.last_seq() > s2);
    }

    /// Protects: found-bug sweep R1 -- two way deletes in a row (through the real
    /// `record_delete`, which `way_delete` calls) are BOTH undoable, newest first,
    /// restoring the list exactly; an older drag below them stays undoable too, and
    /// its redo still works afterwards. Before the fix the prune read the
    /// post-delete fingerprint, so each delete dropped every earlier held entry of
    /// its store: after the second delete only that one could be undone.
    #[test]
    fn two_deletes_in_a_row_both_undo_in_order_and_redo_still_works() {
        let mut h = H::new();
        h.click_commit(mw(ManualWayType::Track, 50.0));
        h.click_commit(mw(ManualWayType::Ancient, 60.0));
        let drag = h.commit_drag(vec![mw(ManualWayType::Road, 10.0)]).expect("a real drag records");
        let before = h.manual.clone();
        assert_eq!(before.len(), 3);

        let d1 = h.delete_real(0);
        let d2 = h.delete_real(0);
        assert_eq!(h.manual, vec![before[2].clone()], "both deletes took their way");
        assert!(h.reversible(d2), "the newest delete is undoable");
        assert!(h.reversible(d1), "the first delete must survive the second one's prune");
        assert!(h.reversible(drag), "and the drag below both of them");
        assert_eq!(h.wu.len(), 3, "nothing held was pruned");

        assert_eq!(h.undo().map(|s| s.label), Some("Delete unnamed ancient".to_string()), "newest first");
        assert_eq!(h.undo().map(|s| s.label), Some("Delete unnamed track".to_string()));
        assert_eq!(h.manual, before, "both ways back, at their own indices");

        assert_eq!(h.undo().map(|s| s.label), Some("Draw road".to_string()), "then the drag");
        assert_eq!(h.manual, before[..2].to_vec());
        assert_eq!(h.redo().as_deref(), Some("Draw road"), "redo still works after the delete undos");
        assert_eq!(h.manual, before);
    }

    /// Protects: the manual list changing under a drag makes its row refuse cleanly (a click-committed way, an edited name): undo returns None, nothing is removed, the row is drawn not reversible and no redo is parked; and a LEDGERED delete above the drag does not break it: the delete undoes first and the drag stays live behind it.
    #[test]
    fn a_changed_manual_list_makes_the_drag_row_refuse() {
        // (a) a way committed by clicking after the drag
        let mut h = H::new();
        let seq = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.click_commit(mw(ManualWayType::Track, 7.0));
        assert!(!h.reversible(seq));
        let before = h.manual.clone();
        assert!(h.undo().is_none());
        assert_eq!(h.manual, before, "nothing removed");
        assert_eq!(h.wu.redo_len(), 0, "a refused undo parks no redo");

        // (b) a way deleted after the drag: the delete is the newest row and
        // undoes first, and only then is the drag live again (chain walked)
        let mut h = H::new();
        h.click_commit(mw(ManualWayType::Track, 7.0));
        let drag = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        let del = h.delete_manual(0);
        assert!(h.reversible(del));
        assert!(h.reversible(drag), "a ledgered delete sits above the drag: the chain is intact, not broken");
        assert_eq!(h.undo().unwrap().label, "Delete unnamed track", "newest row first");
        assert!(h.reversible(drag), "once the delete is undone the list is as the drag left it");
        assert_eq!(h.undo().unwrap().label, "Draw road");

        // (c) the drag's own way renamed: same length, same ends, different name
        let mut h = H::new();
        let seq = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.manual[0].name = "renamed".into();
        assert!(!h.reversible(seq));
        assert!(h.undo().is_none());
        assert_eq!(h.manual[0].name, "renamed");
    }

    /// Protects: the value check behind the fingerprint: an edit the fingerprint does not hash (an interior point, ends and length unchanged) leaves the row live by fingerprint, yet undo refuses without touching the list, drops the held entry and parks no redo.
    #[test]
    fn undo_rechecks_the_slice_value_for_value() {
        let mut h = H::new();
        let seq = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.manual[0].pts[1] = (2.5, 9.0); // interior vertex: not in the fingerprint
        assert!(h.reversible(seq), "premise: the fingerprint cannot see an interior edit");
        let before = h.manual.clone();
        assert!(h.undo().is_none());
        assert_eq!(h.manual, before, "no partial edit");
        assert!(!h.reversible(seq), "the entry was dropped, so the row now reads stale");
        assert_eq!(h.wu.redo_len(), 0);
    }

    /// Protects: a new operation after an undo kills the drag redo: a second drag (which also drops the stack), a click-committed way, a delete and any ledger record each leave the undone drag un-redoable.
    #[test]
    fn a_new_operation_after_an_undo_ends_the_redo() {
        let fresh = || {
            let mut h = H::new();
            h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
            h.undo().unwrap();
            assert!(h.redo_offered().is_some(), "premise");
            h
        };
        let mut h = fresh();
        h.commit_drag(vec![mw(ManualWayType::Track, 5.0)]).unwrap();
        assert_eq!(h.wu.redo_len(), 0, "a new drag drops the stack");
        assert_eq!(h.redo(), None);

        let mut h = fresh();
        h.click_commit(mw(ManualWayType::Track, 5.0));
        assert_eq!(h.redo_offered(), None, "manual store moved");
        assert_eq!(h.redo(), None);
        assert_eq!(h.manual.len(), 1, "and nothing was re-inserted");

        let mut h = fresh();
        h.click_commit(mw(ManualWayType::Track, 5.0));
        h.delete_manual(0); // the list is empty again, but a record happened
        assert_eq!(h.redo_offered(), None, "the delete's record and clear_redo");

        let mut h = fresh();
        h.ledger.record("civ", "Settlement dropped", "", EntryKind::Recorded("no snapshot"));
        assert_eq!(h.redo_offered(), None, "any ledger record is an operation, though the manual list is unchanged");
    }

    /// Protects: the height-depth gate that orders a drag redo against height redos: offered only at the depth the drag was undone at, so a height undo after it hides it until that step is redone.
    #[test]
    fn the_drag_redo_waits_for_the_height_depth_it_was_undone_at() {
        let mut h = H::new();
        h.depth = 2;
        h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.undo().unwrap();
        assert!(h.redo_offered().is_some());
        h.depth = 1; // a height step was undone after the drag
        assert_eq!(h.redo_offered(), None);
        assert_eq!(h.redo(), None);
        assert!(h.manual.is_empty());
        h.depth = 2; // ...and redone again
        assert_eq!(h.redo().as_deref(), Some("Draw road"));
    }

    /// Protects: `restamp_redo`: a height redo's own ledger record keeps an older undone drag offered (it was a redo, not a commit), while an entry with a commit between its undo and the restamped record stays dead.
    #[test]
    fn a_redo_record_restamps_only_the_entries_it_continues() {
        let mut h = H::new();
        h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.undo().unwrap();
        // what `redo_last`'s height branch does around its record
        let before = h.ledger.last_seq();
        h.ledger.record("height", "Sculpt commit", "", EntryKind::HeightSnapshot);
        assert_eq!(h.redo_offered(), None, "without the restamp the record kills the offer");
        h.wu.restamp_redo(before, h.ledger.last_seq());
        assert!(h.redo_offered().is_some(), "with it, the offer survives its own height redo");
        // a genuine commit, then a later redo-style restamp from after it: must not revive
        h.ledger.record("civ", "Settlement dropped", "", EntryKind::Recorded("x"));
        let mid = h.ledger.last_seq();
        h.ledger.record("height", "Sculpt commit", "", EntryKind::HeightSnapshot);
        h.wu.restamp_redo(mid, h.ledger.last_seq());
        assert_eq!(h.redo_offered(), None, "a commit sits between the undo and this restamp: it stays dead");
    }

    /// Protects: a new world clears everything: a Floor record alone kills a parked redo through the ledger stamp and clears the rows, and `WayUndo::clear` (generate/load/clear_undo) drops held entries and the redo stack.
    #[test]
    fn a_new_world_clears_the_drag_rows_and_redo() {
        let mut h = H::new();
        h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.commit_drag(vec![mw(ManualWayType::Track, 4.0)]).unwrap();
        h.undo().unwrap();
        assert!(h.redo_offered().is_some());
        h.ledger.record("world", "Generate world", "", EntryKind::Floor);
        assert_eq!(h.redo_offered(), None);
        assert_eq!(h.labels(), vec!["Generate world"], "the floor cleared every earlier row");
        h.wu.clear();
        assert_eq!(h.wu.redo_len(), 0);
        assert!(h.undo().is_none());
    }

    /// Protects: replacing the world's way list under a held drag (a reload with different ways) can neither corrupt nor panic: the held entry is not live, undo returns None and the new list is untouched.
    #[test]
    fn a_replaced_world_leaves_a_held_drag_inert() {
        let mut h = H::new();
        h.commit_drag(vec![mw(ManualWayType::Road, 1.0), mw(ManualWayType::Road, 2.0)]).unwrap();
        h.manual = vec![mw(ManualWayType::Track, 99.0)]; // shorter, different: even `index..index+n` is out of range
        let before = h.manual.clone();
        assert!(h.undo().is_none());
        assert_eq!(h.manual, before);
    }

    /// Protects: a drag recorded after an unrelated edit prunes the stale entry below it, and only the newest drag's way is removed by the next undo.
    #[test]
    fn stale_entries_are_pruned_and_only_the_newest_drag_undoes() {
        let mut h = H::new();
        let d1 = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        h.click_commit(mw(ManualWayType::Track, 8.0)); // d1 is now stale
        let d2 = h.commit_drag(vec![mw(ManualWayType::Road, 3.0)]).unwrap();
        assert!(!h.reversible(d1));
        assert!(h.reversible(d2));
        assert_eq!(h.undo().unwrap().label, "Draw road");
        assert_eq!(h.manual.len(), 2, "only d2's way was removed");
        assert!(h.undo().is_none(), "d1 is stale: the click way breaks its chain");
    }

    /// Protects: `newest_live_in` (the route `way_stroke_undo` takes) answers per store and per liveness: the newest live manual entry is the drag, another store has none, and a changed manual list yields none.
    #[test]
    fn newest_live_in_is_per_store_and_liveness_aware() {
        let mut h = H::new();
        let d = h.commit_drag(vec![mw(ManualWayType::Road, 1.0)]).unwrap();
        let fps = h.fp();
        assert_eq!(h.wu.newest_live_in(WayStore::Manual, &fps).map(|e| e.seq), Some(d));
        assert_eq!(h.wu.newest_live_in(WayStore::Generated, &fps).map(|e| e.seq), None);
        h.click_commit(mw(ManualWayType::Track, 8.0));
        let fps = h.fp();
        assert_eq!(h.wu.newest_live_in(WayStore::Manual, &fps).map(|e| e.seq), None);
    }
}

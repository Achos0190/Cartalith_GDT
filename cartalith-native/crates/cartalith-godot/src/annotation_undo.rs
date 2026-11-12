//! Undo and redo for the hand-edited annotation stores: map labels, placed
//! icons and landmark names (`EDIT_FROM_CONTEXT_AUDIT.md`, "Undo for label,
//! icon and name edits").
//!
//! ## What it is
//!
//! A third reversible kind beside the height snapshot and the way rows
//! ([`crate::undo::EntryKind::Annotation`]). One ledger row per edit, holding
//! the **whole store** before and after (a clone of a small `Vec`, shared
//! between neighbouring entries by [`Arc`], so a chain of N edits holds N + 1
//! snapshots and not 2N).
//!
//! ## Liveness: the same rule the way rows use, over equality
//!
//! A way row is live while its store's fingerprint is the one it left behind
//! (`context_pick_bridge::WayUndo::live_seqs`). An annotation row is live while
//! the store **equals** the snapshot the edit left behind (`PartialEq` on the
//! stores, cheap and exact, no hash to collide), or, below the newest, equals
//! what the entry above it found. Anything else that changed the store (a
//! regenerate, a load, a pass that adds icons) ends the chain: the row stays in
//! the history drawn as not reversible and nothing is ever written on its
//! behalf. Nothing here persists: like the height and way stacks the history
//! is cleared on generate, load and `clear_undo`, and no project archive
//! carries it (`SAVEFILE_COMPAT.md` has no undo member).
//!
//! ## Redo
//!
//! A LIFO stack of undone entries, offered only for the top and only while
//! nothing was recorded since (the ledger stamp), the height stack is at the
//! depth it had, and the store is exactly as the undo left it. Same gates as
//! `context_pick_bridge::drag_redo_ready`, and for the same reason: a new
//! operation truncates redo everywhere in this ledger.
//!
//! ## Streams
//!
//! A drag, a slider or a resize calls its setter once per sample. The shell
//! brackets such a gesture with [`AnnoUndo::begin_stream`] and
//! [`AnnoUndo::end_stream`]; inside the bracket those setters carry a **stream
//! key**, and while the newest ledger row is this module's entry with the same
//! key, a new sample only replaces that entry's `after`, so the gesture is one
//! step. **Outside a bracket nothing merges**: every call is its own step, which
//! is the safe default for the context card's one-shot Move and Rename.
//!
//! ## Compounds
//!
//! One user action that takes several engine calls (a new label is a create, a
//! reclass and a style set; a paste is a create and a set per label) opens a
//! **compound** with [`AnnoUndo::begin_compound`]: every edit of one store until
//! [`AnnoUndo::end_compound`] folds into the compound's one step, named by the
//! compound, whatever keys the calls carried.
//!
//! The pure logic lives here so it is unit-tested without a live Godot object
//! (`WorldGen` is a cdylib `GodotClass`; see `MISTAKES.md`'s preflight row on
//! probes). `WorldGen`'s methods in `lib.rs` hand it the real stores.

use std::collections::BTreeMap;
use std::sync::Arc;

use cartalith_assets::manual::ManualIcon;
use cartalith_civ::labels::MapLabel;

/// How many annotation edits are held. A labelled judgement, not a
/// measurement: an entry is two shared `Vec` snapshots of a store that holds
/// tens to a few thousand small rows, so this is a sanity bound like
/// `context_pick_bridge::MAX_WAY_UNDO` (64), which it matches.
pub const MAX_ANNO_UNDO: usize = 64;

/// Which hand-edited store an entry belongs to. Fixed order for the liveness walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnoStore {
    /// `LabelBridge::labels`, the hand-placed labels.
    Labels,
    /// `IconEditor::icons`, hand-placed icons and generated landmark glyphs.
    Icons,
    /// `WorldGen::landmark_user_names`.
    Names,
}

impl AnnoStore {
    /// Every store, in the order the liveness walk takes them.
    pub const ALL: [AnnoStore; 3] = [AnnoStore::Labels, AnnoStore::Icons, AnnoStore::Names];

    /// The key `annotation_last_step` reports and the shell matches on
    /// (`"label"` and `"icon"` are the `annotations_removed` kinds).
    pub fn key(self) -> &'static str {
        match self {
            AnnoStore::Labels => "label",
            AnnoStore::Icons => "icon",
            AnnoStore::Names => "name",
        }
    }
}

/// A whole store, cloned.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnoSnap {
    /// The hand-placed labels.
    Labels(Vec<MapLabel>),
    /// The placed icons.
    Icons(Vec<ManualIcon>),
    /// Landmark key to the user's name for it.
    Names(BTreeMap<String, String>),
}

impl AnnoSnap {
    /// Which store this snapshot is of.
    pub fn store(&self) -> AnnoStore {
        match self {
            AnnoSnap::Labels(_) => AnnoStore::Labels,
            AnnoSnap::Icons(_) => AnnoStore::Icons,
            AnnoSnap::Names(_) => AnnoStore::Names,
        }
    }

    /// How many rows the store holds. A different count before and after means
    /// later indices shifted, which is what the shell must be told.
    pub fn len(&self) -> usize {
        match self {
            AnnoSnap::Labels(v) => v.len(),
            AnnoSnap::Icons(v) => v.len(),
            AnnoSnap::Names(m) => m.len(),
        }
    }
}

/// A vault note filed by an element's cell that an edit re-pointed (a move):
/// undo re-points it back. `kind`/`from`/`to` are `reassign_entity`'s arguments for the edit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Repoint {
    /// Label or icon.
    pub kind: cartalith_vault::EntityKind,
    /// The `entity_id` before the edit.
    pub from: i64,
    /// The `entity_id` after the edit.
    pub to: i64,
}

/// One held edit.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnoEntry {
    /// The ledger row's `seq`.
    pub seq: u64,
    /// The history label (`Move label “Ash”`).
    pub label: String,
    /// The store the edit changed.
    pub store: AnnoStore,
    /// The whole store before the edit; what undo writes back.
    pub before: Arc<AnnoSnap>,
    /// The whole store after it; what redo writes and what liveness compares.
    pub after: Arc<AnnoSnap>,
    /// `Some(key)` for a streamed edit (a drag sample), merged while the key repeats.
    pub stream: Option<String>,
    /// A note re-pointing the edit made, reversed by undo.
    pub repoint: Option<Repoint>,
}

/// One undone entry waiting to be redone.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnoRedo {
    /// Identity from `undo::next_step_id`, fresh on every undo: what
    /// `redo_offer_ids` reports, and what orders this offer against an undone
    /// way drag (the larger id was undone later, so it is redone first).
    pub id: u64,
    /// The entry, whole.
    pub entry: AnnoEntry,
    /// `undo.depth()` when it was undone (orders it against the height tail).
    pub height_depth: usize,
    /// `HistoryLedger::last_seq()` at the undo, re-stamped by the redos after
    /// it. A commit since moves the ledger past it and kills the offer.
    pub mark: u64,
}

/// What the last undo or redo of an annotation step did, for the shell's repaint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnnoStep {
    /// The store that changed.
    pub store: AnnoStore,
    /// Whether the store's row count changed, so later indices moved.
    pub shifted: bool,
}

/// The held side of every [`crate::undo::EntryKind::Annotation`] row, plus the redo stack.
#[derive(Debug, Default)]
pub struct AnnoUndo {
    entries: Vec<AnnoEntry>,
    redo: Vec<AnnoRedo>,
    /// True between [`Self::begin_stream`] and [`Self::end_stream`]: a keyed
    /// edit may merge into the gesture's entry.
    stream_open: bool,
    /// `Some(label)` between [`Self::begin_compound`] and [`Self::end_compound`].
    compound: Option<String>,
    /// What the last undo or redo did; `None` before the first.
    last_step: Option<AnnoStep>,
    /// How many undos and redos have happened, ever. **Never reset** (not even by
    /// [`Self::clear`]), so a shell that remembers the last number it saw can tell a
    /// new step from the same one read twice, across a generate or a load.
    step_no: u64,
}

impl AnnoUndo {
    /// An empty history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Drops every entry and every undone entry (generate, load, `clear_undo`).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.redo.clear();
        self.stream_open = false;
        self.compound = None;
        self.last_step = None;
    }

    /// Opens a gesture (pointer press, slider drag start). The newest entry
    /// loses its stream key first, so the gesture's first sample is always a
    /// new step and never merges into an earlier gesture with the same key.
    pub fn begin_stream(&mut self) {
        if let Some(e) = self.entries.last_mut() {
            e.stream = None;
        }
        self.stream_open = true;
    }

    /// Closes the gesture (pointer release, slider drag end).
    pub fn end_stream(&mut self) {
        self.stream_open = false;
    }

    /// Opens a compound named `label`: it is a gesture whose every edit, whatever
    /// its own key, folds into one step carrying this name.
    pub fn begin_compound(&mut self, label: &str) {
        self.begin_stream();
        self.compound = Some(label.to_string());
    }

    /// Closes the compound (and its gesture).
    pub fn end_compound(&mut self) {
        self.compound = None;
        self.stream_open = false;
    }

    /// The open compound's name, if one is open.
    pub fn compound_label(&self) -> Option<&str> {
        self.compound.as_deref()
    }

    /// Whether a gesture is open (the recorder keys an entry only then).
    pub fn is_streaming(&self) -> bool {
        self.stream_open
    }

    /// What the last undo or redo of an annotation step did.
    pub fn last_step(&self) -> Option<AnnoStep> {
        self.last_step
    }

    /// Records what the last undo or redo did, and counts it.
    pub fn set_last_step(&mut self, s: AnnoStep) {
        self.last_step = Some(s);
        self.step_no += 1;
    }

    /// The running count of annotation undos and redos (see the field).
    pub fn step_no(&self) -> u64 {
        self.step_no
    }

    /// How many entries are held (test-only; production asks [`Self::live_seqs`]).
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Every entry that can still be undone, newest first. Per store, walking
    /// newest to oldest: the newest is live when `matches` says the live store
    /// equals its `after`; each older one is live when its `after` equals what
    /// the entry above it found (`before`). The first mismatch ends that
    /// store's chain.
    pub fn live_seqs(&self, matches: &dyn Fn(&AnnoSnap) -> bool) -> Vec<u64> {
        let mut out = Vec::new();
        for store in AnnoStore::ALL {
            let mut above: Option<&AnnoSnap> = None;
            for e in self.entries.iter().rev().filter(|e| e.store == store) {
                let ok = match above {
                    None => matches(&e.after),
                    Some(b) => *e.after == *b,
                };
                if !ok {
                    break;
                }
                out.push(e.seq);
                above = Some(&e.before);
            }
        }
        out.sort_unstable_by(|a, b| b.cmp(a));
        out
    }

    /// The newest live entry.
    pub fn newest_live(&self, matches: &dyn Fn(&AnnoSnap) -> bool) -> Option<&AnnoEntry> {
        let seq = *self.live_seqs(matches).first()?;
        self.entries.iter().find(|e| e.seq == seq)
    }

    /// Takes entry `seq` out; `None` when absent.
    pub fn remove(&mut self, seq: u64) -> Option<AnnoEntry> {
        let pos = self.entries.iter().position(|e| e.seq == seq)?;
        Some(self.entries.remove(pos))
    }

    /// Pushes a held entry, dropping the oldest past [`MAX_ANNO_UNDO`].
    pub fn push(&mut self, e: AnnoEntry) {
        self.entries.push(e);
        if self.entries.len() > MAX_ANNO_UNDO {
            self.entries.remove(0);
        }
    }

    /// Keeps only the entries whose `seq` is in `live` (from [`Self::live_seqs`]).
    /// Their ledger rows stay, drawn as not reversible with [`STALE_REASON`].
    pub fn retain_seqs(&mut self, live: &[u64]) {
        self.entries.retain(|e| live.contains(&e.seq));
    }

    /// The `after` of the newest entry of `store` that equals `pre`, to share
    /// as the next entry's `before` instead of cloning a second copy.
    pub fn shared_before(&self, store: AnnoStore, pre: &AnnoSnap) -> Arc<AnnoSnap> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.store == store)
            .filter(|e| *e.after == *pre)
            .map_or_else(|| Arc::new(pre.clone()), |e| Arc::clone(&e.after))
    }

    /// Merges a streamed sample into the newest entry when it is the same
    /// gesture: a gesture is open, the entry is the ledger's newest row
    /// (`ledger_last`), carries `key`, is of `store`, and its `after` is the
    /// state the sample started from (`pre`). Replaces its `after`. Returns
    /// whether it merged.
    pub fn merge_stream(&mut self, ledger_last: u64, store: AnnoStore, key: &str, pre: &AnnoSnap, after: Arc<AnnoSnap>, repoint_to: Option<i64>) -> bool {
        if !self.stream_open {
            return false;
        }
        let Some(e) = self.entries.last_mut() else { return false };
        if e.seq != ledger_last || e.store != store || e.stream.as_deref() != Some(key) || *e.after != *pre {
            return false;
        }
        e.after = after;
        if let (Some(r), Some(to)) = (e.repoint.as_mut(), repoint_to) {
            r.to = to;
        }
        true
    }

    /// Notes that an edit was recorded: a new operation, so undone entries
    /// can no longer be redone.
    pub fn note_recorded(&mut self) {
        self.redo.clear();
    }

    /// Pushes one undone entry onto the redo stack, dropping the oldest past [`MAX_ANNO_UNDO`].
    pub fn push_redo(&mut self, r: AnnoRedo) {
        self.redo.push(r);
        if self.redo.len() > MAX_ANNO_UNDO {
            self.redo.remove(0);
        }
    }

    /// Takes the top redo entry off the stack.
    pub fn pop_redo(&mut self) -> Option<AnnoRedo> {
        self.redo.pop()
    }

    /// Forgets every undone entry without recording anything (`discard_redo_tail`).
    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }

    /// After a legitimate redo recorded its own ledger row, moves every redo
    /// entry stamped `old` to `new`; entries stamped earlier stay dead.
    pub fn restamp_redo(&mut self, old: u64, new: u64) {
        for r in self.redo.iter_mut().filter(|r| r.mark == old) {
            r.mark = new;
        }
    }

    /// The top undone entry, **only** while the world is exactly as its undo
    /// left it: nothing recorded since (`ledger_last == mark`), the height
    /// stack at the same depth, and the store equal to the entry's `before`
    /// (`matches`). `None` otherwise; a stale offer is never made.
    pub fn redo_ready(&self, ledger_last: u64, height_depth: usize, matches: &dyn Fn(&AnnoSnap) -> bool) -> Option<&AnnoRedo> {
        let top = self.redo.last()?;
        (top.mark == ledger_last && top.height_depth == height_depth && matches(&top.entry.before)).then_some(top)
    }
}

/// The stream key every edit of a compound is recorded under, so they all merge.
pub const COMPOUND_KEY: &str = "compound";

/// The history row's reason for an annotation edit that can no longer be undone.
pub const STALE_REASON: &str =
    "the labels, icons or landmark names were changed outside this history since (a regenerate, a load or a placement pass), so this edit would not fit back";

#[cfg(test)]
mod tests {
    use super::*;

    fn names(pairs: &[(&str, &str)]) -> AnnoSnap {
        AnnoSnap::Names(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
    }

    fn entry(seq: u64, store: AnnoStore, before: AnnoSnap, after: AnnoSnap, stream: Option<&str>) -> AnnoEntry {
        AnnoEntry { seq, label: format!("e{seq}"), store, before: Arc::new(before), after: Arc::new(after), stream: stream.map(str::to_string), repoint: None }
    }

    /// Protects: the liveness chain. The newest entry is live only while the
    /// store equals its `after`; an older one only while it left behind what
    /// the one above found. A store changed outside the history kills the chain.
    #[test]
    fn live_chain_needs_the_store_to_match() {
        let (s0, s1, s2) = (names(&[]), names(&[("a", "A")]), names(&[("a", "AA")]));
        let mut u = AnnoUndo::new();
        u.push(entry(1, AnnoStore::Names, s0.clone(), s1.clone(), None));
        u.push(entry(2, AnnoStore::Names, s1.clone(), s2.clone(), None));
        assert_eq!(u.live_seqs(&|s| *s == s2), vec![2, 1]);
        assert!(u.live_seqs(&|s| *s == names(&[("a", "other")])).is_empty());
        // The newest entry's `before` is not what the older one left behind: only the newest is live.
        let mut v = AnnoUndo::new();
        v.push(entry(1, AnnoStore::Names, s0, s1, None));
        v.push(entry(2, AnnoStore::Names, names(&[("z", "Z")]), s2.clone(), None));
        assert_eq!(v.live_seqs(&|s| *s == s2), vec![2]);
    }

    /// Protects: the newest live entry is the highest seq across stores, and a
    /// dead store's entries never surface.
    #[test]
    fn newest_live_is_the_highest_seq() {
        let icons = |n: usize| {
            AnnoSnap::Icons(vec![
                ManualIcon { x: 0.0, y: 0.0, family: cartalith_assets::manual::ManualIconFamily::Feature, slot: "s".into(), set: None, scale: 1.0, origin: Default::default() };
                n
            ])
        };
        let mut u = AnnoUndo::new();
        u.push(entry(3, AnnoStore::Names, names(&[]), names(&[("k", "v")]), None));
        u.push(entry(5, AnnoStore::Icons, icons(0), icons(1), None));
        let both = |s: &AnnoSnap| *s == names(&[("k", "v")]) || *s == icons(1);
        assert_eq!(u.newest_live(&both).map(|e| e.seq), Some(5));
        let only_names = |s: &AnnoSnap| *s == names(&[("k", "v")]);
        assert_eq!(u.newest_live(&only_names).map(|e| e.seq), Some(3));
    }

    /// Protects: a stream merges only while a gesture is open, into the newest
    /// ledger row with the same key whose `after` is where the sample started;
    /// a closed gesture, a different key, another row on top or a changed start
    /// each refuse, and a new gesture never merges into the previous one.
    #[test]
    fn stream_merges_only_the_same_open_gesture() {
        let (a, b, c) = (names(&[("a", "1")]), names(&[("a", "2")]), names(&[("a", "3")]));
        let mut u = AnnoUndo::new();
        u.push(entry(7, AnnoStore::Names, names(&[]), a.clone(), Some("move")));
        assert!(!u.merge_stream(7, AnnoStore::Names, "move", &a, Arc::new(b.clone()), None), "no gesture is open");
        u.stream_open = true;
        assert!(u.merge_stream(7, AnnoStore::Names, "move", &a, Arc::new(b.clone()), None));
        assert_eq!(*u.entries[0].after, b);
        assert_eq!(*u.entries[0].before, names(&[]), "the gesture keeps its first before");
        assert!(!u.merge_stream(8, AnnoStore::Names, "move", &b, Arc::new(c.clone()), None), "another row is on top");
        assert!(!u.merge_stream(7, AnnoStore::Names, "size", &b, Arc::new(c.clone()), None), "a different key");
        assert!(!u.merge_stream(7, AnnoStore::Names, "move", &a, Arc::new(c.clone()), None), "the sample did not start from the held after");
        u.end_stream();
        assert!(!u.merge_stream(7, AnnoStore::Names, "move", &b, Arc::new(c.clone()), None), "the gesture closed");
        u.begin_stream();
        assert!(u.is_streaming());
        assert!(!u.merge_stream(7, AnnoStore::Names, "move", &b, Arc::new(c), None), "a new gesture starts a new step");
    }

    /// Protects: redo is offered only at the stamp, depth and store state the
    /// undo left, and a restamp keeps an offer alive across a legitimate redo.
    #[test]
    fn redo_gates_on_stamp_depth_and_state() {
        let (s0, s1) = (names(&[]), names(&[("a", "A")]));
        let mut u = AnnoUndo::new();
        u.push_redo(AnnoRedo { id: 9, entry: entry(4, AnnoStore::Names, s0.clone(), s1, None), height_depth: 2, mark: 10 });
        let at_before = |s: &AnnoSnap| *s == s0;
        assert!(u.redo_ready(10, 2, &at_before).is_some());
        assert!(u.redo_ready(11, 2, &at_before).is_none(), "a commit since");
        assert!(u.redo_ready(10, 1, &at_before).is_none(), "the height stack moved");
        assert!(u.redo_ready(10, 2, &|_| false).is_none(), "the store changed");
        u.restamp_redo(10, 12);
        assert!(u.redo_ready(12, 2, &at_before).is_some());
        u.restamp_redo(10, 99);
        assert!(u.redo_ready(12, 2, &at_before).is_some(), "an entry already past the stamp is not moved again");
    }

    /// Protects: a compound folds edits of any key into the one entry its first
    /// edit made, closes with `end_compound`, and a second compound is a new step.
    #[test]
    fn a_compound_folds_every_key_into_one_entry() {
        let (a, b, c) = (names(&[("a", "1")]), names(&[("a", "2")]), names(&[("a", "3")]));
        let mut u = AnnoUndo::new();
        u.begin_compound("Add label");
        assert_eq!(u.compound_label(), Some("Add label"));
        assert!(u.is_streaming());
        u.push(entry(7, AnnoStore::Names, names(&[]), a.clone(), Some(COMPOUND_KEY)));
        assert!(u.merge_stream(7, AnnoStore::Names, COMPOUND_KEY, &a, Arc::new(b.clone()), None));
        assert!(u.merge_stream(7, AnnoStore::Names, COMPOUND_KEY, &b, Arc::new(c.clone()), None));
        assert_eq!(*u.entries[0].after, c);
        assert_eq!(*u.entries[0].before, names(&[]));
        u.end_compound();
        assert_eq!(u.compound_label(), None);
        assert!(!u.is_streaming());
        u.begin_compound("Add label");
        assert!(!u.merge_stream(7, AnnoStore::Names, COMPOUND_KEY, &c, Arc::new(a), None), "a second compound is its own step");
    }

    /// Protects: the step counter rises with every recorded step and survives `clear`, so a shell comparing it never mistakes a new step for an old one.
    #[test]
    fn the_step_counter_counts_and_survives_clear() {
        let mut u = AnnoUndo::new();
        assert_eq!(u.step_no(), 0);
        u.set_last_step(AnnoStep { store: AnnoStore::Labels, shifted: true });
        u.set_last_step(AnnoStep { store: AnnoStore::Names, shifted: false });
        assert_eq!(u.step_no(), 2);
        assert_eq!(u.last_step(), Some(AnnoStep { store: AnnoStore::Names, shifted: false }));
        u.clear();
        assert_eq!(u.step_no(), 2, "a generate must not rewind the counter");
        assert_eq!(u.last_step(), None);
    }

    /// Protects: the bound drops the oldest entry, and `retain_seqs` keeps exactly the live ones.
    #[test]
    fn push_is_bounded_and_prune_keeps_live() {
        let mut u = AnnoUndo::new();
        for i in 0..(MAX_ANNO_UNDO as u64 + 3) {
            u.push(entry(i + 1, AnnoStore::Names, names(&[]), names(&[]), None));
        }
        assert_eq!(u.len(), 64);
        assert_eq!(u.entries[0].seq, 4, "the three oldest went");
        let live = u.live_seqs(&|_| false);
        u.retain_seqs(&live);
        assert_eq!(u.len(), 0);
        let mut v = AnnoUndo::new();
        v.push(entry(1, AnnoStore::Names, names(&[]), names(&[("a", "A")]), None));
        v.push(entry(2, AnnoStore::Labels, AnnoSnap::Labels(vec![]), AnnoSnap::Labels(vec![]), None));
        let live = v.live_seqs(&|s| s.store() == AnnoStore::Labels);
        v.retain_seqs(&live);
        assert_eq!(v.len(), 1);
    }

    /// Protects: `shared_before` shares the previous `after` when it equals the
    /// state an edit starts from, and copies when it does not.
    #[test]
    fn shared_before_reuses_a_matching_after() {
        let s1 = names(&[("a", "A")]);
        let mut u = AnnoUndo::new();
        u.push(entry(1, AnnoStore::Names, names(&[]), s1.clone(), None));
        assert!(Arc::ptr_eq(&u.shared_before(AnnoStore::Names, &s1), &u.entries[0].after));
        assert!(!Arc::ptr_eq(&u.shared_before(AnnoStore::Names, &names(&[("b", "B")])), &u.entries[0].after));
    }
}

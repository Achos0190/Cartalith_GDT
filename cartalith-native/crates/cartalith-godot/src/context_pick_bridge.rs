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

impl WayStore {
    pub const ALL: [WayStore; 3] = [WayStore::Generated, WayStore::SeaLane, WayStore::Manual];

    pub fn key(self) -> &'static str {
        match self {
            WayStore::Generated => "generated",
            WayStore::SeaLane => "sea_lane",
            WayStore::Manual => "manual",
        }
    }

    pub fn parse(s: &str) -> Option<WayStore> {
        match s {
            "generated" => Some(WayStore::Generated),
            "sea_lane" => Some(WayStore::SeaLane),
            "manual" => Some(WayStore::Manual),
            _ => None,
        }
    }
}

/// One deleted way, whole -- whichever store it came from.
#[derive(Debug, Clone, PartialEq)]
pub enum HeldWay {
    Generated(Way),
    SeaLane(SeaRoute),
    Manual(ManualWay),
}

impl HeldWay {
    pub fn store(&self) -> WayStore {
        match self {
            HeldWay::Generated(_) => WayStore::Generated,
            HeldWay::SeaLane(_) => WayStore::SeaLane,
            HeldWay::Manual(_) => WayStore::Manual,
        }
    }
}

/// Mutable views of the three stores, `None` where the store does not exist
/// (no civ layer yet, no infra tools yet).
pub struct WayStores<'a> {
    pub generated: Option<&'a mut Vec<Way>>,
    pub sea: Option<&'a mut Vec<SeaRoute>>,
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
    }
}

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
    pub seq: u64,
    pub index: usize,
    pub way: HeldWay,
    pub fp_before: u64,
    pub fp_after: u64,
}

/// How many deletes are held. A held way is a few hundred points at most, so
/// this is a sanity bound, not a memory budget; the ledger's own
/// [`undo::MAX_LEDGER`] is larger.
pub const MAX_WAY_UNDO: usize = 64;

/// The held side of every [`undo::EntryKind::WayDelete`] ledger row.
#[derive(Debug, Default)]
pub struct WayUndo {
    entries: Vec<WayUndoEntry>,
}

impl WayUndo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, e: WayUndoEntry) {
        self.entries.push(e);
        if self.entries.len() > MAX_WAY_UNDO {
            self.entries.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

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
    Height(String),
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

impl Terminus {
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

    fn way_undo_label(&self, e: &WayUndoEntry) -> String {
        format!("Delete {}", held_way_title(&e.way))
    }

    /// Put the newest live deleted way back and drop its ledger row. Returns
    /// the row's label. `None` when there is no live entry, or when the store
    /// refuses the index (it cannot, while the entry is live; the refusal is
    /// the guard, not a path).
    pub(crate) fn undo_way_step(&mut self) -> Option<String> {
        let fps: Vec<(WayStore, u64)> = WayStore::ALL.iter().map(|&s| (s, self.way_fp(s))).collect();
        let fp = |s: WayStore| fps.iter().find(|(k, _)| *k == s).map_or(0, |(_, v)| *v);
        let seq = self.way_undo.newest_live(&fp)?.seq;
        let e = self.way_undo.remove(seq)?;
        let label = self.way_undo_label(&e);
        let ok = {
            let (civ, infra) = (self.civ.as_mut(), self.infra.as_mut());
            let (generated, sea) = match civ {
                Some(c) => (Some(&mut c.ways), Some(&mut c.sea_routes)),
                None => (None, None),
            };
            let mut st = WayStores { generated, sea, manual: infra.map(|t| &mut t.ways) };
            put_way(&mut st, e.index, e.way)
        };
        self.ledger.remove_seq(seq);
        ok.then_some(label)
    }
}

/// `"road “Old Mill Road”"`, or `"unnamed track"` -- a way described, never
/// given a name it does not have.
fn held_way_title(w: &HeldWay) -> String {
    let (kind, name) = match w {
        HeldWay::Generated(w) => (generated_way_type_key(w.way_type), w.name.as_str()),
        HeldWay::SeaLane(w) => ("sea_lane", w.name.as_str()),
        HeldWay::Manual(w) => (crate::infra_tools_bridge::way_type_key(w.way_type), w.name.as_str()),
    };
    let kind = kind.replace('_', " ");
    if name.is_empty() { format!("unnamed {kind}") } else { format!("{kind} “{name}”") }
}

fn to_v2(p: (f64, f64)) -> Vector2 {
    Vector2::new(p.0 as f32, p.1 as f32)
}

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
        let title = held_way_title(&way);
        let detail = match store {
            WayStore::Generated => "generated network -- Generate roads rebuilds it",
            WayStore::SeaLane => "generated sea lane -- Generate roads rebuilds it",
            WayStore::Manual => "hand-drawn",
        };
        let seq = self.ledger.record("civ", format!("Delete {title}"), detail, undo::EntryKind::WayDelete);
        let fps: Vec<(WayStore, u64)> = WayStore::ALL.iter().map(|&s| (s, self.way_fp(s))).collect();
        self.way_undo.prune_stale(&|s| fps.iter().find(|(k, _)| *k == s).map_or(0, |(_, v)| *v));
        self.way_undo.push(WayUndoEntry { seq, index: i, way, fp_before, fp_after });
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

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::tools::ManualWayType;
    use cartalith_civ::WayType;

    fn way(tid: u64, pts: Vec<(f64, f64)>, t: WayType) -> Way {
        Way { tid, pts, brks: vec![], km: 12.5, name: format!("W{tid}"), way_type: t, a_idx: 0, b_idx: 1, hidden: false }
    }

    fn manual(pts: Vec<(f64, f64)>) -> ManualWay {
        ManualWay { pts, brks: vec![], km: 3.0, sea: false, way_type: ManualWayType::Track, name: String::new(), hidden: false }
    }

    // -- pick ------------------------------------------------------------

    /// A horizontal and a vertical way crossing near (10, 10); the pointer at
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

    /// A break at index 2 means the stroke `pts[1]`-`pts[2]` is not drawn, so
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

    #[test]
    fn a_one_point_run_is_hit_at_its_point() {
        let p = [(5.0, 5.0)];
        let c: Vec<(&[(f64, f64)], &[usize])> = vec![(&p, &[])];
        let (_, d, at) = nearest_polyline(&c, (5.0, 5.6), 1.0).unwrap();
        assert!((d - 0.6).abs() < 1e-12);
        assert_eq!(at, (5.0, 5.0));
    }

    // -- delete / undo -----------------------------------------------------

    fn fps(g: &[Way], s: &[SeaRoute], m: &[ManualWay]) -> impl Fn(WayStore) -> u64 {
        let (a, b, c) = (fp_generated(Some(g)), fp_sea(Some(s)), fp_manual(Some(m)));
        move |st| match st {
            WayStore::Generated => a,
            WayStore::SeaLane => b,
            WayStore::Manual => c,
        }
    }

    /// Delete the middle generated way, then put it back: the list is
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

    /// Two deletes from one store are both live and undo newest first; after
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

    /// Chains are per store: a manual delete stays live when the generated
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

    /// Same ends, same length, same name -- a re-routed interior is still a
    /// different list, and a held way must not be re-inserted into it.
    #[test]
    fn the_fingerprint_sees_a_rerouted_interior() {
        let a = vec![way(1, vec![(0.0, 0.0), (5.0, 5.0)], WayType::Road)];
        let b = vec![way(1, vec![(0.0, 0.0), (2.0, 4.0), (5.0, 5.0)], WayType::Road)];
        assert_ne!(fp_generated(Some(&a)), fp_generated(Some(&b)));
        assert_ne!(fp_generated(Some(&a)), fp_generated(None), "no store is not an empty store");
        assert_eq!(fp_generated(Some(&a)), fp_generated(Some(&a.clone())));
    }

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

    #[test]
    fn a_held_way_is_described_not_named() {
        assert_eq!(held_way_title(&HeldWay::Manual(manual(vec![]))), "unnamed track");
        assert_eq!(held_way_title(&HeldWay::Generated(way(4, vec![], WayType::Highway))), "highway “W4”");
    }

    // -- river -------------------------------------------------------------

    /// 6x4, no wrap. A channel runs along row 1 from x=1 to x=4; x=5 row 1
    /// is sea.
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

    /// From x=2 the chain runs 8 -> 9 -> 10 -> 11 (sea), 3 steps: the branch
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

    /// Cut the channel at x=3 (cell 9 has no channel receiver): the walk
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

    #[test]
    fn a_seam_step_is_one_cell_on_a_wrapping_world() {
        // 6 wide: cell 5 (x=5) to cell 0 (x=0) on row 0.
        assert_eq!(branch_cells_length(&[5, 0], 6, true), 1.0);
        assert_eq!(branch_cells_length(&[5, 0], 6, false), 5.0);
    }

    /// The channel receiver wins where there is one; the D8 one fills the rest.
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

    /// A 3x3 tree: every cell drains to the centre (4) except 0, which drains
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

//! The planar street graph — reference lines 28363-28512 (`makeGraph` …
//! `edgeBetween`), plus `extractFaces`, which is what makes blocks possible at
//! all.
//!
//! This is the milestone where the crate's index design gets settled, and the
//! answer is: **keep the reference's dense integer ids with tombstones**. The
//! JS uses `g.nodes[i]` / `g.edges[i]` with `id === index` and a soft-delete
//! `alive` flag, and the *stability* of those ids is load-bearing —
//! [`Graph::split_edge`] leaves the split edge in place, dead, and later
//! milestones read `e.alive` while walking `g.edges` by index. A slotmap or a
//! generational arena would be the "better" Rust answer and would change the
//! iteration order that `extract_faces` and every `filter(e => e.alive)` pass
//! depends on. So: `Vec` with tombstones, and ids that are never reused.
//!
//! **Ordering is load-bearing throughout.** Three separate places depend on it:
//!
//! - [`Graph::edges_near`] returns a JS `Set`, which iterates in *insertion*
//!   order. [`Graph::attach_point`] picks its best edge with a strict `<`, so
//!   the first candidate wins a tie; [`Graph::add_street`] sorts its crossing
//!   list with JS's *stable* sort, so equal parameters keep that same order.
//!   The port therefore returns an order-preserving `Vec`, not a `HashSet`.
//! - Crossing hits are collected before on-segment node hits, so a crossing and
//!   a node at the same parameter `t` keep the crossing first.
//! - `extract_faces`' outer-face pick uses a strict `>` over `|area|`, so on a
//!   tie the *lowest-indexed* face is the outer one. A two-face graph whose
//!   faces have equal absolute area really does mark face 0 as outer, and a
//!   golden test pins exactly that.
//!
//! **Every distance here goes through [`crate::geom::js_hypot`]**, never
//! `f64::hypot`. That is not a stylistic preference: [`Graph::attach_point`]'s
//! 11 m snap, [`Graph::raw_edge`]'s 3.5 m minimum and [`Graph::nearest_node`]'s
//! search radius are all *threshold* comparisons against V8's one-ulp-high
//! `Math.hypot`, and being more accurate than the reference flips them.
//!
//! **Encapsulation finding**, verified by grep across all 2,937 lines of block
//! 4 rather than assumed: `cell`, `grid`, `nextE` and `nextN` are touched
//! **only** by the functions in this module. No later milestone reaches into
//! the spatial index. `nodes`, `edges` and `adj` are read widely (always as
//! `n.adj.filter(id => g.edges[id].alive)`), which is why they are public here.

use crate::geom::{Vec2, dist_pt_seg, js_atan2, js_hypot, js_or, poly_area, seg_int};
use std::collections::HashMap;

/// A street-graph node. `id` always equals the index into [`Graph::nodes`] —
/// the reference's `g.nextN++` and `g.nodes.push(n)` cannot diverge — but it is
/// kept as a field because `adj` stores ids and the port should read like the
/// source it is checked against.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    /// Incident edge ids, live and dead alike. Callers filter on `alive`; the
    /// reference does so at every one of its ~10 read sites.
    pub adj: Vec<usize>,
}

impl Node {
    /// The node as a point, for the geometry kernel.
    pub fn pt(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// A street-graph edge. Soft-deleted, never removed: `id` stability is what
/// lets `adj` lists and the spatial index hold plain integers.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: usize,
    pub a: usize,
    pub b: usize,
    /// `'primary'`, `'street'`, `'lane'`, `'quay'`, `'ringroad'`, … — kept as
    /// `&'static str` rather than promoted to an enum. The reference compares
    /// it by string in six places and `hashModel` serialises it verbatim, so
    /// the string *is* the value. Enumerating the variants now would also mean
    /// guessing at classes later milestones introduce, which is exactly the
    /// running-ahead this port avoids.
    pub cls: &'static str,
    /// Carriageway width in metres.
    pub w: f64,
    pub epoch: i32,
    /// Human-readable provenance, shown in the reference's UI and built by
    /// string concatenation at several call sites, so it cannot be `&'static
    /// str`. Never hashed and never compared — metadata only.
    pub prov: String,
    pub alive: bool,
}

/// One face found by [`Graph::extract_faces`] — a candidate town block.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub node_ids: Vec<usize>,
    pub poly: Vec<Vec2>,
    /// Signed shoelace area. Interior faces come out negative under this
    /// traversal and the outer boundary positive, which is why the outer face
    /// is picked by *absolute* area rather than by sign.
    pub area: f64,
    pub outer: bool,
}

/// The planar street graph (`makeGraph`, reference line 28363).
///
/// The reference's `nextN`/`nextE` counters are not stored: they are
/// unconditionally `nodes.len()` and `edges.len()`, since every increment is
/// paired with a `push` and nothing is ever removed. A golden test asserts that
/// equivalence against the reference's own counters rather than leaving it as
/// a claim.
#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// Uniform-grid cell size in metres. The reference hard-codes 26 and never
    /// writes it again.
    pub cell: f64,
    /// Uniform spatial index: cell -> edge ids, in insertion order. The
    /// reference keys it with the string `cx + ':' + cy`; an `(i64, i64)` key
    /// is the same partition, and the map is only ever probed by key — its own
    /// iteration order is never used, so no ordering is lost.
    pub grid: HashMap<(i64, i64), Vec<usize>>,
    /// The reference's `g._fromPaths`, a **dynamic** JS property rather than
    /// part of `makeGraph`'s literal.
    ///
    /// `buildPrimariesFromPaths` sets it (line 28830) when it lays at least one
    /// injected real-road primary, and `builtMassHull` (line 29709, milestone
    /// 10) reads it to discount the bare degree-2 vertices a ~55 m resampled
    /// road drags in — without which the enceinte over-encloses along
    /// arterials. Milestone 2 deliberately left it out because nothing set or
    /// read it yet; milestone 6 is the milestone that sets it.
    ///
    /// `makeGraph` leaves it absent, and `undefined` is falsy, so `false` is the
    /// faithful initial value.
    pub from_paths: bool,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// `makeGraph()` — an empty graph with the reference's 26 m index cell.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            cell: 26.0,
            grid: HashMap::new(),
            from_paths: false,
        }
    }

    /// `gridCellsForSeg` (line 28365), collected rather than called back.
    ///
    /// The reference passes a callback because `indexEdge` needs to mutate the
    /// map it is walking; collecting the keys first is the same visit sequence
    /// with the same first-seen de-duplication, and it keeps `&mut self` out of
    /// a closure. Walks the segment at 0.7 cell steps and dilates each sample
    /// by its 3×3 neighbourhood, so an edge is registered in every cell it can
    /// possibly be queried from.
    fn cells_for_seg(&self, a: Vec2, b: Vec2) -> Vec<(i64, i64)> {
        let steps = (a.dist(b) / (self.cell * 0.7)).ceil().max(1.0);
        let steps_i = steps as i64;
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for i in 0..=steps_i {
            let p = a.lerp(b, i as f64 / steps);
            let cx = (p.x / self.cell).floor() as i64;
            let cy = (p.y / self.cell).floor() as i64;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let k = (cx + dx, cy + dy);
                    if seen.insert(k) {
                        out.push(k);
                    }
                }
            }
        }
        out
    }

    /// The two endpoints of an edge, as points.
    fn ends(&self, eid: usize) -> (Vec2, Vec2) {
        let e = &self.edges[eid];
        (self.nodes[e.a].pt(), self.nodes[e.b].pt())
    }

    /// `indexEdge` (line 28372).
    fn index_edge(&mut self, eid: usize) {
        let (a, b) = self.ends(eid);
        for k in self.cells_for_seg(a, b) {
            self.grid.entry(k).or_default().push(eid);
        }
    }

    /// `unindexEdge` (line 28374) — removes the *first* occurrence per cell,
    /// exactly as `indexOf`/`splice` does. An edge is pushed at most once per
    /// cell, so first-occurrence removal is total.
    fn unindex_edge(&mut self, eid: usize) {
        let (a, b) = self.ends(eid);
        for k in self.cells_for_seg(a, b) {
            if let Some(arr) = self.grid.get_mut(&k)
                && let Some(i) = arr.iter().position(|&id| id == eid)
            {
                arr.remove(i);
            }
        }
    }

    /// `edgesNear` (line 28376) — every edge id indexed in any cell the segment
    /// `a`→`b` touches, **in first-seen order**.
    ///
    /// The reference returns a `Set`, and JS `Set` iteration is insertion
    /// order; two callers ([`Self::attach_point`]'s first-wins `<` and
    /// [`Self::add_street`]'s stable sort) depend on it, so this returns an
    /// ordered `Vec` rather than a `HashSet`.
    pub(crate) fn edges_near(&self, a: Vec2, b: Vec2) -> Vec<usize> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for k in self.cells_for_seg(a, b) {
            if let Some(arr) = self.grid.get(&k) {
                for &id in arr {
                    if seen.insert(id) {
                        out.push(id);
                    }
                }
            }
        }
        out
    }

    /// `n.adj.filter(id => g.edges[id].alive)` — the incident edges of `nid`
    /// that are still alive, **in `adj` order**.
    ///
    /// The reference writes this filter at every one of its ~10 read sites and
    /// never removes an edge, only tombstones it (see [`Edge::alive`]), so
    /// "degree" always means the live degree. Order matters: `grow`'s
    /// continuation picks `live_adj(n).len() == 1`'s single edge and
    /// `built_mass_hull` indexes the list, so this must not become a set.
    pub fn live_adj(&self, nid: usize) -> impl Iterator<Item = usize> + '_ {
        self.nodes[nid].adj.iter().copied().filter(|&id| self.edges[id].alive)
    }

    /// `n.adj.filter(id => g.edges[id].alive).length`.
    ///
    /// Six modules had open-coded this before the milestone-15 reconciliation
    /// pass, one of them (`cleanup`) as a private function of its own.
    pub fn live_degree(&self, nid: usize) -> usize {
        self.live_adj(nid).count()
    }

    /// `addNode` (line 28379).
    fn add_node(&mut self, x: f64, y: f64) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { id, x, y, adj: Vec::new() });
        id
    }

    /// `nearestNode` (line 28380) — the closest node strictly within `r`,
    /// searched over the grid cells within `ceil(r/cell)+1` of `(x, y)`.
    ///
    /// Only nodes reachable through a **live** edge are candidates: the
    /// reference scans the index, and a node whose every edge is dead (or which
    /// never got one, see [`Self::add_street`]'s orphan note) is invisible here.
    pub fn nearest_node(&self, x: f64, y: f64, r: f64) -> Option<usize> {
        let mut best = None;
        let mut bd = r;
        let cx = (x / self.cell).floor() as i64;
        let cy = (y / self.cell).floor() as i64;
        let cr = (r / self.cell).ceil() as i64 + 1;
        let mut seen = std::collections::HashSet::new();
        for dx in -cr..=cr {
            for dy in -cr..=cr {
                let Some(arr) = self.grid.get(&(cx + dx, cy + dy)) else { continue };
                for &id in arr {
                    let Some(e) = self.edges.get(id) else { continue };
                    if !e.alive {
                        continue;
                    }
                    for nid in [e.a, e.b] {
                        if !seen.insert(nid) {
                            continue;
                        }
                        let n = &self.nodes[nid];
                        let d = js_hypot(n.x - x, n.y - y);
                        if d < bd {
                            bd = d;
                            best = Some(nid);
                        }
                    }
                }
            }
        }
        best
    }

    /// `rawEdge` (line 28390) — create an edge without any planarity work.
    ///
    /// Three rejections, all the reference's: identical endpoints, an existing
    /// live edge between the pair (returned as-is, *not* duplicated), and a
    /// span under 3.5 m.
    fn raw_edge(
        &mut self,
        a_id: usize,
        b_id: usize,
        cls: &'static str,
        w: f64,
        epoch: i32,
        prov: &str,
    ) -> Option<usize> {
        if a_id == b_id {
            return None;
        }
        for &eid in &self.nodes[a_id].adj {
            let e = &self.edges[eid];
            if e.alive && (e.a == b_id || e.b == b_id) {
                return Some(eid);
            }
        }
        if self.nodes[a_id].pt().dist(self.nodes[b_id].pt()) < 3.5 {
            return None;
        }
        let id = self.edges.len();
        self.edges.push(Edge {
            id,
            a: a_id,
            b: b_id,
            cls,
            w,
            epoch,
            prov: prov.to_string(),
            alive: true,
        });
        self.nodes[a_id].adj.push(id);
        self.nodes[b_id].adj.push(id);
        self.index_edge(id);
        Some(id)
    }

    /// `splitEdge` (line 28397) — insert a node into an existing edge.
    ///
    /// The edge is tombstoned rather than removed, and two fresh edges inherit
    /// its class, width, epoch and provenance. Returns an existing endpoint
    /// instead when `pt` lands within 3.5 m of one, which is why callers must
    /// treat the return as "the node you should connect to", not "the new node".
    fn split_edge(&mut self, eid: usize, pt: Vec2) -> usize {
        let (e_a, e_b) = (self.edges[eid].a, self.edges[eid].b);
        let (pa, pb) = (self.nodes[e_a].pt(), self.nodes[e_b].pt());
        if pt.dist(pa) < 3.5 {
            return e_a;
        }
        if pt.dist(pb) < 3.5 {
            return e_b;
        }
        self.edges[eid].alive = false;
        self.unindex_edge(eid);
        for nid in [e_a, e_b] {
            // The reference splices at `indexOf(e.id)` **unguarded**, and JS
            // `splice(-1, 1)` drops the LAST element — so a miss would silently
            // corrupt the adjacency instead of throwing. It cannot miss:
            // `raw_edge` pushes the id onto both endpoints and nothing removes
            // it while the edge is alive. Reproduced rather than hardened, with
            // the note that milestone 11's `_killEdge` guards the identical
            // splice with `if (k >= 0)` — the reference is inconsistent here,
            // and that inconsistency is worth seeing rather than smoothing.
            let adj = &mut self.nodes[nid].adj;
            match adj.iter().position(|&i| i == eid) {
                Some(k) => {
                    adj.remove(k);
                }
                None => {
                    adj.pop();
                }
            }
        }
        let (cls, w, epoch, prov) = {
            let e = &self.edges[eid];
            (e.cls, e.w, e.epoch, e.prov.clone())
        };
        let n = self.add_node(pt.x, pt.y);
        self.raw_edge(e_a, n, cls, w, epoch, &prov);
        self.raw_edge(n, e_b, cls, w, epoch, &prov);
        n
    }

    /// `attachPoint` (line 28407) — bind a loose point to the network: snap to a
    /// node within 11 m, else split the nearest edge within 9 m (making a
    /// T-junction), else place a new isolated node.
    ///
    /// The split parameter is clamped to `[0.03, 0.97]`, which keeps a
    /// T-junction off an existing endpoint even when the perpendicular foot
    /// would land on one — [`Self::split_edge`]'s 3.5 m guard then catches the
    /// cases the clamp does not.
    fn attach_point(&mut self, x: f64, y: f64) -> usize {
        const SNAP: f64 = 11.0;
        const ESNAP: f64 = 9.0;
        if let Some(n) = self.nearest_node(x, y, SNAP) {
            return n;
        }
        let p = Vec2::new(x, y);
        let mut best_e = None;
        let mut bd = ESNAP;
        let mut bpt = Vec2::default();
        for eid in self.edges_near(p, p) {
            let Some(e) = self.edges.get(eid) else { continue };
            if !e.alive {
                continue;
            }
            let (a, b) = self.ends(eid);
            let d = dist_pt_seg(p, a, b);
            if d < bd {
                let ab = b - a;
                // JS `|| 1`: a zero (or NaN) squared length becomes 1.
                let l2 = js_or(ab.x * ab.x + ab.y * ab.y, 1.0);
                let t = (((x - a.x) * ab.x + (y - a.y) * ab.y) / l2).clamp(0.03, 0.97);
                bd = d;
                best_e = Some(eid);
                bpt = Vec2::new(a.x + ab.x * t, a.y + ab.y * t);
            }
        }
        match best_e {
            Some(eid) => self.split_edge(eid, bpt),
            None => self.add_node(x, y),
        }
    }

    /// `addStreet` (line 28422) — **the planarity invariant**. Attaches both
    /// endpoints, splits every live edge the new segment crosses, promotes every
    /// existing node lying within 2.5 m of the segment's interior to a junction,
    /// then chains the whole ordered sequence together.
    ///
    /// Returns the ids of the edges actually created (which may be fewer than
    /// the chain length: [`Self::raw_edge`] drops sub-3.5 m links).
    ///
    /// Note the reference leaves **orphan nodes** behind: if `attach_point`
    /// creates two fresh nodes and every resulting link is then rejected as too
    /// short, the nodes stay in `g.nodes` with empty `adj`. That is reference
    /// behaviour, it is visible in the goldens, and later passes tolerate it by
    /// always filtering on live adjacency.
    // Nine arguments, matching `addStreet(g, ax, ay, bx, by, cls, w, epoch,
    // prov)` position for position. Bundling the four street attributes into a
    // spec struct would please clippy and would make the ~14 call sites the
    // later milestones bring across stop looking like the reference lines they
    // are checked against. Kept literal; the type system already separates the
    // one pair that could plausibly be transposed (`w: f64` vs `epoch: i32`).
    #[allow(clippy::too_many_arguments)]
    pub fn add_street(
        &mut self,
        ax: f64,
        ay: f64,
        bx: f64,
        by: f64,
        cls: &'static str,
        w: f64,
        epoch: i32,
        prov: &str,
    ) -> Vec<usize> {
        let na = self.attach_point(ax, ay);
        let nb = self.attach_point(bx, by);
        if na == nb {
            return Vec::new();
        }
        let a = self.nodes[na].pt();
        let b = self.nodes[nb].pt();
        let near_ids = self.edges_near(a, b);

        // Crossings first — so that a crossing and an on-segment node sharing a
        // parameter keep the crossing ahead of the node under the stable sort.
        //
        // The `1e-4` interior guards below (and the `1e-3` pair in the node loop)
        // are **redundant inside this engine's own site box**, which milestone 2
        // established by mutation rather than by inspection: loosening either to
        // `1e-9` changes no golden. A crossing at `t = 1e-4` sits `1e-4 · L` from
        // an endpoint, so it only escapes `split_edge`'s 3.5 m fold-back when
        // `L > 35 km`; the node guard likewise only matters past `L > 3.5 km`.
        // `SITE_WM`/`SITE_HM` are 1700 × 1250 m, a 2.1 km diagonal. Kept as
        // written — they are the reference's, and they are what stops the
        // degenerate case if a later milestone ever runs this on a larger box.
        let mut hits: Vec<(f64, Hit)> = Vec::new();
        for &eid in &near_ids {
            let Some(e) = self.edges.get(eid) else { continue };
            if !e.alive || e.a == na || e.b == na || e.a == nb || e.b == nb {
                continue;
            }
            let (ea, eb) = self.ends(eid);
            if let Some(h) = seg_int(a, b, ea, eb)
                && h.t > 1e-4
                && h.t < 1.0 - 1e-4
                && h.u > 1e-4
                && h.u < 1.0 - 1e-4
            {
                hits.push((h.t, Hit::Cross { pt: h.pt, eid }));
            }
        }

        // Existing nodes lying on the new segment's interior must become
        // junctions too, or the graph stops being planar the moment a street is
        // laid along an earlier one.
        let mut seen_n: std::collections::HashSet<usize> = [na, nb].into_iter().collect();
        let ab = b - a;
        let abl2 = js_or(ab.x * ab.x + ab.y * ab.y, 1.0);
        for &eid in &near_ids {
            let Some(e) = self.edges.get(eid) else { continue };
            if !e.alive {
                continue;
            }
            for nid in [e.a, e.b] {
                if !seen_n.insert(nid) {
                    continue;
                }
                let nd = self.nodes[nid].pt();
                if dist_pt_seg(nd, a, b) < 2.5 {
                    let t = ((nd.x - a.x) * ab.x + (nd.y - a.y) * ab.y) / abl2;
                    if t > 1e-3 && t < 1.0 - 1e-3 {
                        hits.push((t, Hit::Node(nid)));
                    }
                }
            }
        }

        // JS `Array.prototype.sort` is stable (spec-mandated since ES2019), and
        // so is `sort_by`, so on a tie the collection order above survives.
        //
        // **A tie turns out to be unreachable**, which is worth stating because
        // milestone 2 spent real effort trying to construct one. Two crossings
        // at the same `t` would be the same point, so the two edges intersect
        // there and were already split into a shared node whose half-edges the
        // `1e-4` guard then excludes. Two on-segment nodes at the same `t` lie
        // on one perpendicular within 2.5 m of the segment, hence within 5 m of
        // each other, and `attach_point`'s 11 m snap prevents the second from
        // existing. A crossing tied with a node means the crossing point is the
        // node's own foot, at most 2.5 m away, which `split_edge`'s 3.5 m guard
        // folds back into that node. So the reference's three guards conspire
        // to make `t` values distinct. The stable sort stays because it is what
        // the reference specifies and because that argument is a property of
        // the current constants, not of the algorithm.
        hits.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut chain = vec![na];
        for (_, h) in hits {
            match h {
                Hit::Node(nid) => chain.push(nid),
                Hit::Cross { pt, eid } => {
                    if !self.edges[eid].alive {
                        continue;
                    }
                    let n = self.split_edge(eid, pt);
                    chain.push(n);
                }
            }
        }
        chain.push(nb);

        let mut made = Vec::new();
        for pair in chain.windows(2) {
            if let Some(e) = self.raw_edge(pair[0], pair[1], cls, w, epoch, prov) {
                made.push(e);
            }
        }
        made
    }

    /// `addPolylineStreet` (line 28455) — [`Self::add_street`] over consecutive
    /// pairs, concatenating what each produced.
    pub fn add_polyline_street(
        &mut self,
        pts: &[Vec2],
        cls: &'static str,
        w: f64,
        epoch: i32,
        prov: &str,
    ) -> Vec<usize> {
        let mut out = Vec::new();
        for pair in pts.windows(2) {
            out.extend(self.add_street(pair[0].x, pair[0].y, pair[1].x, pair[1].y, cls, w, epoch, prov));
        }
        out
    }

    /// `edgeBetween` (line 28509) — the live edge joining two nodes, if any.
    pub fn edge_between(&self, a_id: usize, b_id: usize) -> Option<usize> {
        self.nodes[a_id].adj.iter().copied().find(|&eid| {
            let e = &self.edges[eid];
            e.alive && (e.a == b_id || e.b == b_id)
        })
    }

    /// `extractFaces` (line 28462) — planar face extraction by angularly-sorted
    /// half-edge traversal. This is what turns a street network into blocks.
    ///
    /// Each live edge contributes two half-edges; each traversal repeatedly
    /// takes the *previous* entry in the destination's angle-sorted incidence
    /// list, which walks a face boundary. Dead-end spurs come back on
    /// themselves and are collapsed by the `v, w, v` stack rule, so a stub
    /// hanging off a closed loop does not corrupt the loop's polygon.
    ///
    /// The 20,000-step guard is the reference's; a traversal that hits it is
    /// **discarded**, not truncated. Note the exact arithmetic: the JS
    /// `while (guard++ < 20000)` leaves `guard` at 20001 when the bound is what
    /// stopped it, and at the step count when the loop closed, so the
    /// post-check `guard >= 20000` also rejects a face that closed on step
    /// 20000 exactly. Reproduced as written.
    pub fn extract_faces(&self) -> Vec<Face> {
        struct HalfEdge {
            eid: usize,
            other: usize,
            ang: f64,
        }
        let live: Vec<usize> =
            self.edges.iter().filter(|e| e.alive).map(|e| e.id).collect();
        let mut adj_sorted: Vec<Vec<HalfEdge>> =
            (0..self.nodes.len()).map(|_| Vec::new()).collect();
        for &eid in &live {
            let e = &self.edges[eid];
            for (from, to) in [(e.a, e.b), (e.b, e.a)] {
                let a = &self.nodes[from];
                let b = &self.nodes[to];
                adj_sorted[from].push(HalfEdge {
                    eid,
                    other: to,
                    // `js_atan2`, not `f64::atan2`. This is the **sort key**
                    // the face traversal walks, so one ulp reorders two edges
                    // leaving a node in nearly the same direction and the
                    // traversal then produces a different city block --
                    // `JS_SEMANTICS_AUDIT.md` §4.4 named it "the argmax hazard
                    // in a different costume" and the next one to fix. Rust and
                    // V8 disagree on 17-23 % of ordinary `atan2` arguments, the
                    // largest divergence in the workspace. Milestone 2 wrote
                    // this before that was measured; all 19 of its golden
                    // scenarios still pass unmodified afterwards, which is the
                    // proof that the fix moved nothing they can see (their
                    // incidences are too far apart for a last bit to reorder).
                    ang: js_atan2(b.y - a.y, b.x - a.x),
                });
            }
        }
        for arr in &mut adj_sorted {
            arr.sort_by(|p, q| p.ang.partial_cmp(&q.ang).unwrap_or(std::cmp::Ordering::Equal));
        }

        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut faces: Vec<Face> = Vec::new();
        let he_id = |eid: usize, dir: usize| eid * 2 + dir;

        for &eid0 in &live {
            for dir in [0usize, 1usize] {
                let h0 = he_id(eid0, dir);
                if visited.contains(&h0) {
                    continue;
                }
                let mut seq: Vec<usize> = Vec::new();
                let mut h = h0;
                let mut guard = 0usize;
                loop {
                    let before = guard;
                    guard += 1;
                    if before >= 20000 {
                        break;
                    }
                    visited.insert(h);
                    let ee = &self.edges[h >> 1];
                    let d = h & 1;
                    let (from, to) = if d == 1 { (ee.b, ee.a) } else { (ee.a, ee.b) };
                    seq.push(from);
                    let arr = &adj_sorted[to];
                    // JS `indexOf`-style miss yields -1, and the reference then
                    // indexes `(-1 - 1 + len) % len`. Unreachable given the
                    // adjacency invariant, but reproduced rather than asserted.
                    let idx = arr
                        .iter()
                        .position(|he| he.eid == ee.id && he.other == from)
                        .map_or(-1isize, |i| i as isize);
                    let len = arr.len() as isize;
                    let nxt = &arr[(((idx - 1 + len) % len + len) % len) as usize];
                    let ne = &self.edges[nxt.eid];
                    h = he_id(ne.id, if ne.a == to { 0 } else { 1 });
                    if h == h0 {
                        break;
                    }
                }
                if guard >= 20000 {
                    continue;
                }
                // Collapse spurs: a dead-end round trip shows up as `v, w, v`.
                let mut stack: Vec<usize> = Vec::new();
                for v in seq {
                    if stack.last() == Some(&v) {
                        continue;
                    }
                    if stack.len() >= 2 && stack[stack.len() - 2] == v {
                        stack.pop();
                    } else {
                        stack.push(v);
                    }
                }
                while stack.len() >= 3 && stack[0] == stack[stack.len() - 1] {
                    stack.pop();
                }
                if stack.len() < 3 {
                    continue;
                }
                let poly: Vec<Vec2> = stack.iter().map(|&id| self.nodes[id].pt()).collect();
                let area = poly_area(&poly);
                faces.push(Face { node_ids: stack, poly, area, outer: false });
            }
        }
        if faces.is_empty() {
            return faces;
        }
        // Strict `>`, so the lowest-indexed face wins a tie for outermost.
        let mut outer = 0usize;
        let mut mx = -1.0f64;
        for (i, f) in faces.iter().enumerate() {
            if f.area.abs() > mx {
                mx = f.area.abs();
                outer = i;
            }
        }
        faces[outer].outer = true;
        faces
    }
}

/// What `addStreet` found on the new segment: a crossing to split, or an
/// existing node to promote to a junction.
enum Hit {
    Cross { pt: Vec2, eid: usize },
    Node(usize),
}

#[cfg(test)]
mod tests;

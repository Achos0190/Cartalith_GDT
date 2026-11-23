//! Measures of a laid-out town against its water, shared by the ignored
//! censuses (`town_river_census`) and by the Ruling HM census a live
//! `WorldGen` answers (`bank_site::town_bank_census`): buildings standing in
//! water and the share of the built river bank a building or quay fronts.
//!
//! Moved here unchanged from `town_river_census.rs` (2026-11-23), which is a
//! test-only module and so cannot be reached by a `#[func]`. Measurements
//! only: nothing here feeds generation.

/// **Ruling FQ** (owner, 2026-11-16): the context the layout `l` was laid
/// out from: `ctx` itself, or, where `ctx` is a stretched town's whose
/// stretched layout `run_layout` refused
/// (`cartalith_urban::stretch::stretch_holds`), its reference-box `plain`
/// context, whose water raster is the one `l`'s metres index.
pub(crate) fn laid_ctx<'a>(
    ctx: &'a cartalith_civ::urban_adapter::UrbanContext,
    l: &cartalith_civ::urban_adapter::UrbanLayout,
) -> &'a cartalith_civ::urban_adapter::UrbanContext {
    match (ctx.site_box, ctx.plain.as_deref()) {
        (Some((bw, bh)), Some(plain)) if (l.wm - bw).abs() > 0.5 || (l.hm - bh).abs() > 0.5 => plain,
        _ => ctx,
    }
}

/// Distance from point `p` to segment `a`-`b`, in the units of the inputs.
pub(crate) fn seg_dist(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
}

/// Distance from `p` to polyline `pl` (at least two points), or `None` for a
/// shorter line: "no line" is not a distance.
pub(crate) fn line_dist(p: (f64, f64), pl: &[(f64, f64)]) -> Option<f64> {
    if pl.len() < 2 {
        return None;
    }
    Some(pl.windows(2).map(|w| seg_dist(p, w[0], w[1])).fold(f64::INFINITY, f64::min))
}

/// Does segment `a`-`b` cross segment `c`-`d` (proper or touching)?
pub(crate) fn segs_cross(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    let o = |p: (f64, f64), q: (f64, f64), r: (f64, f64)| (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
    let (d1, d2, d3, d4) = (o(c, d, a), o(c, d, b), o(a, b, c), o(a, b, d));
    (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0)
}

/// Distance from `p` to the nearest wet cell of a layout's water mask
/// (`WaterCtx::mask`, the 22 m raster the City Viewer draws as the town's
/// water), measured to the cell's square, not its centre: 0 inside a wet
/// cell. Scans a window of `reach` cells; `None` when no wet cell lies in it
/// ("no water near" is not a distance).
/// `m` is the mask as `(cells, mw, mh, cell_m)`; the godot crate cannot name
/// `cartalith_urban::WaterCtx`, so the census passes its fields.
pub(crate) fn mask_dist(m: (&[u8], usize, usize, f64), p: (f64, f64), reach: i64) -> Option<f64> {
    let (mask, mw, mh, c) = m;
    let (ci, cj) = ((p.0 / c).floor() as i64, (p.1 / c).floor() as i64);
    let mut best = f64::INFINITY;
    for j in (cj - reach).max(0)..=(cj + reach).min(mh as i64 - 1) {
        for i in (ci - reach).max(0)..=(ci + reach).min(mw as i64 - 1) {
            if mask[j as usize * mw + i as usize] != 1 {
                continue;
            }
            let (x0, y0) = (i as f64 * c, j as f64 * c);
            let dx = (x0 - p.0).max(0.0).max(p.0 - (x0 + c));
            let dy = (y0 - p.1).max(0.0).max(p.1 - (y0 + c));
            best = best.min((dx * dx + dy * dy).sqrt());
        }
    }
    best.is_finite().then_some(best)
}

/// Even-odd point-in-polygon.
pub(crate) fn in_poly(p: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut c = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            c = !c;
        }
    }
    c
}

/// How far from the bank, metres, a building or a quay's water side must be
/// for a stretch of the built reach to count as fronted
/// ([`bank_frontage`]): two street widths ([`BANK_STREET_W`]), the same
/// line as the judgement's first waterfront bar.
pub(crate) const FRONTED_M: f64 = 2.0 * BANK_STREET_W;

/// How far inland a bank point's nearest building may be for that point to
/// lie in the town's **built reach** ([`bank_frontage`]), metres. A labelled
/// judgement: `EMBANKMENT_REACH` (46 m, the reference's harbour back-street
/// depth) rounded up, so the reach is the bank the town's own streets come
/// down to.
pub(crate) const BUILT_REACH_M: f64 = 50.0;

/// The street width the bank-frontage shares are counted in, metres: the
/// widest street `grow` lays (`r.range(3.4, 5.4)`), the figure `_framegap_probe`
/// and `framegap.txt` used ("2 street widths, 10.8 m"), so the two read alike.
pub(crate) const BANK_STREET_W: f64 = 5.4;

/// **The waterfront's frontage share** (2026-11-15, `TOWN_GENERATOR_JUDGEMENT.md`
/// change 5's third bar, *"at least 70 % of the built reach's bank length
/// fronted by a building or a quay"*). The centreline is walked every 8 m
/// inside the box; each sample gives a bank point on each side
/// (`river_w / 2` out along the local normal). A bank point is in the built
/// reach when a building centre on its side lies within [`BUILT_REACH_M`],
/// and fronted when a building vertex or a `quay` edge's water side lies
/// within [`FRONTED_M`] of it. Returns `(fronted, fronted or with any
/// street's water side within FRONTED_M, in reach)` sample counts: the second
/// also counts a riverside `street`, `lane` or `primary` (Ruling EZ item 1's
/// *"a street in between the river and house"*, a quay street by another
/// class). A measurement only.
pub(crate) fn bank_frontage(l: &cartalith_civ::urban_adapter::UrbanLayout, river: &[(f64, f64)]) -> (usize, usize, usize) {
    let (mut fronted, mut fronted_st, mut reach) = (0usize, 0usize, 0usize);
    if river.len() < 2 {
        return (0, 0, 0);
    }
    let cents: Vec<(f64, f64)> = l.buildings.iter().map(|b| {
        let c = cartalith_civ::urban_adapter::poly_centroid(&b.poly);
        (c.x, c.y)
    }).collect();
    let verts: Vec<(f64, f64)> = l.buildings.iter().flat_map(|b| b.poly.iter().map(|v| (v.x, v.y))).collect();
    let quays: Vec<((f64, f64), (f64, f64), f64)> =
        l.edges.iter().filter(|e| e.cls == "quay").map(|e| ((e.a.x, e.a.y), (e.b.x, e.b.y), e.w)).collect();
    let streets: Vec<((f64, f64), (f64, f64), f64)> = l
        .edges
        .iter()
        .filter(|e| matches!(e.cls, "street" | "lane" | "primary"))
        .map(|e| ((e.a.x, e.a.y), (e.b.x, e.b.y), e.w))
        .collect();
    let inside = |q: (f64, f64)| q.0 >= 0.0 && q.1 >= 0.0 && q.0 <= l.wm && q.1 <= l.hm;
    for w in river.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        if len < 1e-9 {
            continue;
        }
        let (dx, dy) = ((b.0 - a.0) / len, (b.1 - a.1) / len);
        let steps = (len / 8.0).ceil() as usize;
        for k in 0..steps {
            let t = k as f64 * len / steps as f64;
            let c = (a.0 + dx * t, a.1 + dy * t);
            if !inside(c) {
                continue;
            }
            for s in [1.0, -1.0] {
                let q = (c.0 - dy * s * l.river_w / 2.0, c.1 + dx * s * l.river_w / 2.0);
                let on_side = |p: (f64, f64)| ((p.0 - c.0) * -dy + (p.1 - c.1) * dx) * s > 0.0;
                let near_b = cents.iter().any(|&p| on_side(p) && (p.0 - q.0).hypot(p.1 - q.1) <= BUILT_REACH_M);
                if !near_b {
                    continue;
                }
                reach += 1;
                let by_bld = verts.iter().any(|&p| on_side(p) && (p.0 - q.0).hypot(p.1 - q.1) <= FRONTED_M);
                let by_quay = quays.iter().any(|&(qa, qb, qw)| seg_dist(q, qa, qb) - qw / 2.0 <= FRONTED_M && on_side(((qa.0 + qb.0) / 2.0, (qa.1 + qb.1) / 2.0)));
                fronted += usize::from(by_bld || by_quay);
                let by_street = streets.iter().any(|&(qa, qb, qw)| seg_dist(q, qa, qb) - qw / 2.0 <= FRONTED_M && on_side(((qa.0 + qb.0) / 2.0, (qa.1 + qb.1) / 2.0)));
                fronted_st += usize::from(by_bld || by_quay || by_street);
            }
        }
    }
    (fronted, fronted_st, reach)
}

/// Buildings of layout `l` standing in water, by the riverbank audit's three
/// tests (see [`riverbank_gap_audit`]): on a river layout, a footprint the
/// river channel reaches or crosses (centreline within half `river_w`), a sample (vertex, edge
/// midpoint, centroid) in a sea or lake cell of the town's open-water raster
/// (`open`, the mask before the river band is stamped), or a sample inside
/// `water_poly`. A building counted by two tests is counted once. Zero is the
/// bar with `urban_riverbank` on.
pub(crate) fn wet_buildings(l: &cartalith_civ::urban_adapter::UrbanLayout, open: Option<(&[u8], usize, usize, f64)>) -> usize {
    let river: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
    let wpoly: Vec<(f64, f64)> = l.water_poly.iter().map(|v| (v.x, v.y)).collect();
    let is_river = l.site_kind == "river" || l.site_kind == "riverthrough";
    let mut wet = 0usize;
    for b in &l.buildings {
        let poly: Vec<(f64, f64)> = b.poly.iter().map(|v| (v.x, v.y)).collect();
        let n = poly.len();
        let mut samples = poly.clone();
        let (mut cx, mut cy) = (0.0, 0.0);
        for k in 0..n {
            samples.push(((poly[k].0 + poly[(k + 1) % n].0) / 2.0, (poly[k].1 + poly[(k + 1) % n].1) / 2.0));
            cx += poly[k].0 / n as f64;
            cy += poly[k].1 / n as f64;
        }
        samples.push((cx, cy));
        // Only a river's polyline is a channel. On a `coast` or `bay` layout it
        // is `shore_from_mask`'s shoreline: land cells beside the sea sorted
        // along one axis, which zigzags across the town; the 457 "wet"
        // buildings this test counted in six `bay` towns of seed 483920 at
        // 2 048 were houses within 6 m of that line, on dry land (0 in a sea
        // or lake cell, 0 in `water_poly`; `both_banks_audit`, 2026-11-15).
        let channel = is_river
            && (river.windows(2).any(|rw| (0..n).any(|k| segs_cross(rw[0], rw[1], poly[k], poly[(k + 1) % n])))
                || samples.iter().any(|&q| line_dist(q, &river).is_some_and(|d| d < l.river_w / 2.0)));
        let sea = open.is_some_and(|o| samples.iter().any(|&q| mask_dist(o, q, 0) == Some(0.0)));
        let in_poly_ = wpoly.len() >= 3 && samples.iter().any(|&q| in_poly(q, &wpoly));
        wet += usize::from(channel || sea || in_poly_);
    }
    wet
}

/// How far apart [`footprint_over`] samples a footprint's lines, metres.
/// **A labelled judgement**: under the narrowest lane the layout lays (3 m
/// wide streets are 4 to 5.4 m) and the smallest house side, so a line with
/// an end on each bank of the narrowest drawn river (12 m) cannot step over it.
pub(crate) const OVER_STEP_M: f64 = 4.0;

/// How near a wall's wet run a water gate must stand for the run to count as
/// a gated river wall ([`footprint_over`]), metres. `fortify::build_wall`
/// puts a water gate where the circuit meets the river and refuses a second
/// within 40 m; half that separates two crossings, and 15 m is under it.
/// **A labelled judgement.**
pub(crate) const GATE_NEAR_M: f64 = 15.0;

/// How near a street edge a detected crossing must lie for the edge to carry
/// it ([`footprint_over`]), metres. `water::detect_river_crossings` computes
/// the point on the edge itself, so this is only rounding room.
pub(crate) const BRIDGE_ON_EDGE_M: f64 = 1.5;

/// How far inside the water a quay may reach and still be on the waterline
/// ([`footprint_over`]), metres. A quay is built along the water's edge
/// (`buildHarbour`, and the generator's own water sweeps exempt it), so its
/// line touches the stroke; **a labelled judgement**, 2 m: under half its own
/// width (4 to 6 m). A quay deeper than that is a street in the river.
/// Against a lake or the sea, where the depth is not known, a quay is never
/// counted.
pub(crate) const QUAY_ON_WATERLINE_M: f64 = 2.0;

/// [`footprint_over`]'s counts for one town against one kind of water.
#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Over {
    /// Buildings with their centre or a point of their outline in the water, and buildings tested.
    pub b: usize,
    pub of_b: usize,
    /// Wall-ring runs in the water with no water gate on them; runs with one; ring segments tested.
    pub w: usize,
    pub w_gated: usize,
    pub of_w: usize,
    /// Street edges in the water that are no part of a bridge; edges that are; edges tested.
    pub s: usize,
    pub s_bridge: usize,
    pub of_s: usize,
    /// The longest bridge (the wet edges of one chain, summed), metres.
    pub span_m: f64,
    /// The longest bridge's length over the water alone (each chain edge's length times the share of its
    /// [`OVER_STEP_M`] samples in the water), metres: the measure the generator's ferry rule reads
    /// (`cartalith_urban::cleanup::BRIDGE_MAX_SPAN_M`, Ruling HQ).
    pub wet_span_m: f64,
    /// The classes of the edges counted in `s`, as `class count` pairs, for a cause dump.
    pub s_by: String,
    /// How deep in the water the deepest counted point of the buildings, the wall runs and the street edges is
    /// (`depth`'s answer), and about how long the longest counted wall run is, metres. For a cause dump.
    pub depth_m: [f64; 3],
    pub w_run_m: f64,
}

/// **Ruling HN: what of town `l` lies over water**, where `depth` answers how deep
/// a local point (metres, the layout's frame) is in the water measured (the
/// map's drawn river stroke, its drawn lake, or its drawn sea; metres inside
/// its edge, any number where that is not known), `None` for a point out of it. Lines are
/// sampled every [`OVER_STEP_M`], ends included, so a line with an end on each
/// bank is seen.
///
/// With `exempt`, the two things a river town may lay over its river are
/// counted apart, and defined here:
///
/// - **a real bridge**: a chain of street edges in the water, joined end to
///   end, one of which carries a crossing the layout itself detected
///   (`UrbanLayout::bridges`, `water::detect_river_crossings`: a live non-quay
///   road meeting the river's centreline, within [`BRIDGE_ON_EDGE_M`] of the
///   edge). A wet edge joined to no such chain is a street in the river.
/// - **a gated river wall**: a run of the wall ring in the water with one of
///   the wall's own water gates (`Gate::water`) within [`GATE_NEAR_M`] of it.
///   A wet run with no gate is a wall in the river.
///
/// No span limit: the stroke is as wide as the map draws it, which on a
/// coarse grid is a cell (2 km at 384 cells over 800 km); the longest bridge
/// is reported (`span_m`), not judged. Without `exempt` (lake and sea) every
/// wet edge and run counts.
///
/// The ring is the wall the shell draws (`wall_ring`). A measurement: nothing
/// here feeds generation.
pub(crate) fn footprint_over(l: &cartalith_civ::urban_adapter::UrbanLayout, depth: &dyn Fn((f64, f64)) -> Option<f64>, exempt: bool) -> Over {
    let pt = |v: cartalith_civ::urban_adapter::Vec2| (v.x, v.y);
    let buildings: Vec<Vec<(f64, f64)>> = l.buildings.iter().map(|b| b.poly.iter().map(|&v| pt(v)).collect()).collect();
    let ring: Vec<(f64, f64)> = l.wall.ring.iter().flatten().map(|&v| pt(v)).collect();
    let gates: Vec<(f64, f64)> = l.wall.gates.iter().filter(|g| g.water).map(|g| pt(g.pt)).collect();
    let edges: Vec<((f64, f64), (f64, f64), &str)> = l.edges.iter().map(|e| (pt(e.a), pt(e.b), e.cls)).collect();
    let bridges: Vec<(f64, f64)> = l.bridges.iter().flatten().map(|b| pt(b.pt)).collect();
    footprint_over_parts(&buildings, &ring, &gates, &edges, &bridges, depth, exempt)
}

/// [`footprint_over`] on plain geometry (local metres): building outlines, the wall ring (empty: none), the wall's
/// water gates, street edges as `(a, b, class)` and the detected crossings. Split out so the rule is tested on
/// hand-made shapes (`tests::the_footprint_measure_counts_what_the_ruling_forbids`).
pub(crate) fn footprint_over_parts(
    buildings: &[Vec<(f64, f64)>],
    ring: &[(f64, f64)],
    gates: &[(f64, f64)],
    edges: &[((f64, f64), (f64, f64), &str)],
    bridges: &[(f64, f64)],
    depth: &dyn Fn((f64, f64)) -> Option<f64>,
    exempt: bool,
) -> Over {
    // Deepest sample of each kind's counted (not exempt) footprint.
    let deep = std::cell::Cell::new([0.0f64; 3]);
    let note = |kind: usize, d: f64| {
        let mut v = deep.get();
        v[kind] = v[kind].max(d);
        deep.set(v);
    };
    let wet = |p: (f64, f64)| depth(p).is_some();
    let deepest = |pts: &mut dyn Iterator<Item = (f64, f64)>| pts.filter_map(depth).fold(0.0, f64::max);
    let samples = |a: (f64, f64), b: (f64, f64)| {
        let n = ((a.0 - b.0).hypot(a.1 - b.1) / OVER_STEP_M).ceil().max(1.0) as usize;
        (0..=n).map(move |i| (a.0 + (b.0 - a.0) * i as f64 / n as f64, a.1 + (b.1 - a.1) * i as f64 / n as f64))
    };
    let mut o = Over { of_b: buildings.len(), of_s: edges.len(), ..Over::default() };
    for poly in buildings {
        let n = poly.len();
        if n == 0 {
            continue;
        }
        let c = (poly.iter().map(|v| v.0).sum::<f64>() / n as f64, poly.iter().map(|v| v.1).sum::<f64>() / n as f64);
        let over = wet(c) || (0..n).any(|k| samples(poly[k], poly[(k + 1) % n]).any(wet));
        o.b += usize::from(over);
        if over {
            note(0, deepest(&mut (0..n).flat_map(|k| samples(poly[k], poly[(k + 1) % n])).chain(std::iter::once(c))));
        }
    }
    if ring.len() >= 2 {
        o.of_w = ring.len();
        // The ring's samples in order, each segment's far end left to the next segment.
        let pts: Vec<(f64, f64)> = (0..ring.len()).flat_map(|k| {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            let v: Vec<_> = samples(a, b).collect();
            v[..v.len() - 1].to_vec()
        }).collect();
        let w: Vec<bool> = pts.iter().map(|&p| wet(p)).collect();
        let n = pts.len();
        // Start after a dry sample so no run is split by the list's seam; a ring wholly wet is one run.
        let start = (0..n).find(|&i| !w[i]).map_or(0, |i| i + 1);
        let mut run: Vec<(f64, f64)> = Vec::new();
        for step in 0..=n {
            let i = (start + step) % n;
            if step < n && w[i] {
                run.push(pts[i]);
            } else if !run.is_empty() {
                let gated = exempt && run.iter().any(|&p| gates.iter().any(|&g| (p.0 - g.0).hypot(p.1 - g.1) <= GATE_NEAR_M));
                if gated {
                    o.w_gated += 1
                } else {
                    o.w += 1;
                    note(1, deepest(&mut run.iter().copied()));
                    o.w_run_m = o.w_run_m.max(run.len() as f64 * OVER_STEP_M);
                }
                run.clear();
            }
        }
    }
    // A quay stands on the waterline by design: it is in the water only past [`QUAY_ON_WATERLINE_M`].
    let wet_edge: Vec<bool> = edges.iter().map(|e| samples(e.0, e.1).any(|p| depth(p).is_some_and(|d| e.2 != "quay" || d > QUAY_ON_WATERLINE_M))).collect();
    // Chains of wet edges by shared end (ends keyed to the centimetre), flooded from the edges that carry a crossing.
    let key = |v: (f64, f64)| ((v.0 * 100.0).round() as i64, (v.1 * 100.0).round() as i64);
    let mut at: std::collections::HashMap<(i64, i64), Vec<usize>> = std::collections::HashMap::new();
    for (i, e) in edges.iter().enumerate().filter(|(i, _)| wet_edge[*i]) {
        at.entry(key(e.0)).or_default().push(i);
        at.entry(key(e.1)).or_default().push(i);
    }
    let mut chain = vec![usize::MAX; edges.len()];
    let crossings: &[(f64, f64)] = if exempt { bridges } else { &[] };
    let mut spans: Vec<f64> = Vec::new();
    let mut wet_spans: Vec<f64> = Vec::new();
    let wet_len = |a: (f64, f64), b: (f64, f64)| {
        let v: Vec<_> = samples(a, b).collect();
        (a.0 - b.0).hypot(a.1 - b.1) * v.iter().filter(|&&p| wet(p)).count() as f64 / v.len() as f64
    };
    for (i, e) in edges.iter().enumerate() {
        if !wet_edge[i] || chain[i] != usize::MAX || !crossings.iter().any(|&b| seg_dist(b, e.0, e.1) <= BRIDGE_ON_EDGE_M) {
            continue;
        }
        let id = spans.len();
        spans.push(0.0);
        wet_spans.push(0.0);
        let mut stack = vec![i];
        chain[i] = id;
        while let Some(j) = stack.pop() {
            spans[id] += (edges[j].0.0 - edges[j].1.0).hypot(edges[j].0.1 - edges[j].1.1);
            wet_spans[id] += wet_len(edges[j].0, edges[j].1);
            for end in [key(edges[j].0), key(edges[j].1)] {
                for &k in at.get(&end).into_iter().flatten() {
                    if chain[k] == usize::MAX {
                        chain[k] = id;
                        stack.push(k);
                    }
                }
            }
        }
    }
    o.span_m = spans.iter().copied().fold(0.0, f64::max);
    o.wet_span_m = wet_spans.iter().copied().fold(0.0, f64::max);
    let mut by: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (i, e) in edges.iter().enumerate().filter(|(i, _)| wet_edge[*i]) {
        if chain[i] == usize::MAX {
            o.s += 1;
            *by.entry(e.2).or_default() += 1;
            note(2, deepest(&mut samples(e.0, e.1)));
        } else {
            o.s_bridge += 1;
        }
    }
    o.depth_m = deep.get();
    o.s_by = by.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(" ");
    o
}

/// How near a town's market a street must come to count as one of the market's own
/// ([`streets_near`]), metres. **A labelled judgement**: Ruling HN's finding on the
/// owner's town was 641 streets and none within 150 m of its market (the rampart
/// sweep had kept the far bank's piece only), so 150 m is the figure that miss was
/// read at.
pub(crate) const MARKET_STREET_NEAR_M: f64 = 150.0;

/// How many of town `l`'s street edges come within `r` metres of `p` (the exact
/// point-to-segment distance). For the census's gate on a town that kept no
/// street round its market (Ruling HQ). A measurement; nothing here feeds generation.
pub(crate) fn streets_near(l: &cartalith_civ::urban_adapter::UrbanLayout, p: cartalith_civ::urban_adapter::Vec2, r: f64) -> usize {
    l.edges.iter().filter(|e| seg_dist((p.x, p.y), (e.a.x, e.a.y), (e.b.x, e.b.y)) <= r).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects Ruling HN's measure (`footprint_over_parts`) on a river along `y = 0`, 40 m wide (water where
    /// `|y| < 20`): of two houses the one with a corner 2 m in the water counts; of a ring whose south side runs
    /// through the water in two places, the run with a water gate on it is a gated river wall and the other counts;
    /// of the streets, a three-piece road that crosses the river with the detected crossing on its middle piece is a
    /// bridge (all three pieces, 120 m of them, the two outer ones reaching into the water from each bank), a street that dips into the water without crossing counts, and a
    /// street on land does not. A line with an end on each bank and no vertex in the water is still seen (the
    /// sampling). Without the exemptions (lake and sea) every wet run and street counts. MUTATION: `GATE_NEAR_M`
    /// 15 to 1500 gates both runs; `BRIDGE_ON_EDGE_M` 1.5 to 1500 makes the dipping street a bridge;
    /// `OVER_STEP_M` 4 to 400 misses the wall's crossings.
    #[test]
    fn the_footprint_measure_counts_what_the_ruling_forbids() {
        let depth = |p: (f64, f64)| (p.1.abs() < 20.0).then(|| 20.0 - p.1.abs());
        let sq = |x: f64, y: f64| vec![(x, y), (x + 10.0, y), (x + 10.0, y + 10.0), (x, y + 10.0)];
        let buildings = [sq(0.0, 30.0), sq(40.0, 12.0)];
        // A box north of the river with two prongs down across it: each prong's two sides cross the water.
        let ring = [(-300.0, 200.0), (300.0, 200.0), (300.0, 50.0), (210.0, 50.0), (210.0, -50.0), (190.0, -50.0), (190.0, 50.0), (-190.0, 50.0), (-190.0, -50.0), (-210.0, -50.0), (-210.0, 50.0), (-300.0, 50.0)];
        let gates = [(190.0, 0.0), (210.0, 0.0)];
        let edges = [
            ((100.0, 60.0), (100.0, 10.0), "primary"),
            ((100.0, 10.0), (100.0, -10.0), "primary"),
            ((100.0, -10.0), (100.0, -60.0), "primary"),
            ((120.0, 60.0), (120.0, 15.0), "street"),
            ((0.0, 60.0), (90.0, 60.0), "street"),
            // A quay along the waterline, 1 m in: on it. A second, 6 m in: a street in the river.
            ((-100.0, 19.0), (-40.0, 19.0), "quay"),
            ((-100.0, 14.0), (-40.0, 14.0), "quay"),
        ];
        let bridges = [(100.0, 0.0)];
        let o = footprint_over_parts(&buildings, &ring, &gates, &edges, &bridges, &depth, true);
        assert_eq!((o.b, o.of_b), (1, 2), "one house in the water");
        assert_eq!((o.w, o.w_gated), (2, 2), "the east prong's two crossings are gated, the west prong's are not");
        assert_eq!((o.s, o.s_bridge, o.of_s), (2, 3, 7), "the dipping street and the deep quay count; the bridge's three pieces and the quay on the waterline do not");
        assert_eq!(o.s_by, "quay 1 street 1");
        assert!((o.span_m - 120.0).abs() < 1e-9, "the bridge road's three pieces, summed: {}", o.span_m);
        // Ruling HQ: over the water alone, about the river's 40 m (the samples' spacing rounds it up a little).
        assert!(o.wet_span_m > 38.0 && o.wet_span_m < 45.0, "the bridge over the water: {}", o.wet_span_m);
        assert!((o.depth_m[0] - 8.0).abs() < 1e-9 && (o.depth_m[2] - 6.0).abs() < 1e-9, "deepest house point 8 m, street (the quay) 6 m: {:?}", o.depth_m);
        let open = footprint_over_parts(&buildings, &ring, &gates, &edges, &bridges, &depth, false);
        assert_eq!((open.b, open.w, open.w_gated, open.s, open.s_bridge), (1, 4, 0, 5, 0), "no exemption over a lake or the sea");
    }
}

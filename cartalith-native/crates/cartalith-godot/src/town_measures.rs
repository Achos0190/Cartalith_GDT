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

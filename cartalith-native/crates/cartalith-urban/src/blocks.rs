//! Milestone 12 — blocks and parcels (`buildBlocks`, `buildParcels`, reference
//! lines 30193-30344).
//!
//! This is the milestone that turns a street *network* into buildable *land*.
//! [`build_blocks`] takes the planar faces milestone 2 already extracts and
//! insets each by half the width of the street fronting it, giving the block's
//! buildable interior; [`build_parcels`] plats that interior into strip lots by
//! the reference's vertex-bisector method — the first stage in this whole
//! subsystem whose output is building-sized rather than town-sized.
//!
//! ## Why this milestone came before 8-11
//!
//! Out of order on purpose, and the order is worth stating rather than leaving
//! to look like an oversight. The City Viewer needed discrete, colourable
//! shapes to render a town as anything other than a wire diagram, and parcels
//! are the smallest stage that produces them. Every dependency this milestone
//! has was already built and golden-tested at milestone 1-2 — [`ensure_ccw`],
//! [`inset_poly`], [`poly_centroid`], [`point_in_poly`], [`poly_area`],
//! [`poly_self_intersects`], [`seg_int`], `Graph::edge_between`,
//! `Graph::extract_faces` and the `logn`/`chance`/`range` RNG draws — so
//! nothing here is a new primitive. Two functions, no new kernel.
//!
//! ## What the input graph is missing, and what that costs
//!
//! The reference reaches `buildBlocks` with a graph that has been through
//! `buildPlaza` (milestone 8), `lanePass` and `removeWaterCrossings`
//! (milestone 11). None of those exist yet, and each leaves a mark here:
//!
//! - **No plaza.** `buildPlaza` runs on the organic branch too (reference line
//!   31024), not only the radial one, and its output is what
//!   [`build_blocks`]'s `plaza` argument marks as unbuilt. Passing [`None`] is
//!   faithful to *this* function's contract — it is exactly what the reference
//!   does when there is no plaza — but the town it draws therefore has no open
//!   market square, and the block over the market anchor gets platted like any
//!   other. That is a real visible gap, not a styling choice.
//! - **No `removeWaterCrossings`.** Streets may still cross the channel, so
//!   faces can straddle water. This milestone's own guards absorb most of it:
//!   [`build_blocks`] drops a block whose inset centroid is wet, and
//!   [`build_parcels`] rejects a lot with *any* corner in the water (the
//!   reference's own footprint test, not a centroid test).
//! - **No `lanePass`.** Fewer lanes means larger, coarser faces than the
//!   reference would produce from the same seed.
//!
//! Milestones 8 and 11 will change what comes out of here without changing
//! anything in this file.

use crate::geom::{
    Vec2, ensure_ccw, inset_poly, js_max, js_min, point_in_poly, poly_area, poly_centroid,
    poly_self_intersects, seg_int,
};
use crate::graph::Graph;
use crate::rng::{Substream, fnv1a, stream};
use crate::rules::{DEFAULT_RULES, Rules};
use crate::site::Site;

/// A plaza, as [`build_blocks`] reads one. Milestone 8's `buildPlaza` is what
/// produces it; nothing in this port does yet, so every caller passes [`None`]
/// — see this module's header for what that costs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plaza {
    pub center: Vec2,
}

/// One town block — a face of the street graph, inset to its buildable
/// interior.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// `'blk' + n`, the reference's own id, and the seed of the per-block RNG
    /// substream [`build_parcels`] draws from. Kept as a string because that
    /// string *is* what gets hashed.
    pub id: String,
    /// The buildable interior: `face_poly` inset by each fronting street's
    /// half-width plus a 1.4 m verge. CCW.
    pub poly: Vec<Vec2>,
    /// The face itself, before the inset.
    pub face_poly: Vec<Vec2>,
    /// Node ids of `face_poly`, wound to match it.
    pub face_ids: Vec<usize>,
    /// Per-edge inset distance actually used.
    pub edge_dists: Vec<f64>,
    pub area: f64,
    /// Kept unbuilt. Always `false` in this port — there is no plaza to test
    /// against (module header).
    pub plaza: bool,
}

/// One strip parcel — a street-fronting lot, and the smallest thing this
/// subsystem generates.
///
/// The polygon is always the reference's own `[P0, P1, Q1, Q0]` winding:
/// **`poly[0]`/`poly[1]` are the street frontage** and `poly[3]`/`poly[2]` the
/// back, which is what lets a renderer find the ridge line (the midline
/// between them) without storing it. The reference keeps `F0`/`F1`/`B1`/`B0`
/// as separate fields for milestone 13's convenience; they are the same four
/// points in the same order, so they are not duplicated here.
#[derive(Debug, Clone, PartialEq)]
pub struct Parcel {
    pub id: String,
    pub poly: Vec<Vec2>,
    /// The [`Block::id`] this lot was platted inside.
    pub block: String,
    pub frontage: f64,
    pub depth: f64,
    pub area: f64,
    /// Epochs since the fronting street was laid — older streets are more
    /// subdivided (M-PAR-1).
    pub age: f64,
    pub edge_cls: &'static str,
    /// **This port's own field, not the reference's.** A stable 0..1 scalar
    /// for a renderer to vary a rooftop's brightness and saturation with, so
    /// that a drawn town reads as weathered rather than uniform.
    ///
    /// It is drawn from a **separate** RNG substream (`'roof-tone'`), never
    /// from the per-block `'parcels/…'` stream the geometry comes out of.
    /// That is not a stylistic choice: taking one extra draw from the block
    /// stream would shift every subsequent frontage width and depth, and the
    /// parcels would stop matching the reference's. A separate stream cannot
    /// perturb a sequence it does not share.
    ///
    /// Colour itself is deliberately *not* decided here —
    /// `urban_layout_draw.gd` owns the palette, the same way it already owns
    /// the street and water colours.
    pub tone: f64,
}

/// `buildBlocks` (line 30193) — the faces of the street graph, inset by their
/// own fronting street widths.
///
/// The `120 < |A| < 140_000` m² band is the reference's: below it a face is a
/// junction artefact, above it a face is the un-enclosed hinterland rather than
/// a block. `extract_faces`' outer face is skipped, which milestone 2 pinned as
/// a first-index-wins tie-break on *absolute* area.
pub fn build_blocks(g: &Graph, plaza: Option<Plaza>, site: &Site) -> Vec<Block> {
    let faces = g.extract_faces();
    let mut blocks = Vec::new();
    let mut b_id = 0usize;
    for f in &faces {
        if f.outer {
            continue;
        }
        let a = f.area.abs();
        // Clippy suggests `!(120.0..=140_000.0).contains(&a)` here. **Do not
        // take it**: the two differ on NaN. The reference writes
        // `if(A<120||A>140000)continue;`, and both comparisons are false for a
        // NaN area, so a NaN face is *kept*; `!contains` inverts a false into
        // a true and would skip it instead. `poly_area` can only produce NaN
        // from a NaN vertex, which milestones 8-11 are exactly the sort of
        // stage to introduce, so the difference is not hypothetical.
        #[allow(clippy::manual_range_contains, reason = "NaN semantics differ; see above")]
        if a < 120.0 || a > 140_000.0 {
            continue;
        }
        let poly = ensure_ccw(&f.poly);
        // `ensureCCW` reverses the polygon when the signed area is negative;
        // the ids must be reversed on exactly the same test so `poly[i]` and
        // `ids[i]` stay the same vertex.
        let ids: Vec<usize> = if poly_area(&f.poly) >= 0.0 {
            f.node_ids.clone()
        } else {
            f.node_ids.iter().rev().copied().collect()
        };
        let n = ids.len();
        let mut dists = Vec::with_capacity(n);
        for i in 0..n {
            let e = g.edge_between(ids[i], ids[(i + 1) % n]);
            // A missing edge takes the reference's own 4 m default rather than
            // being skipped -- the face came from the graph, so the only way
            // here is a wound-past-a-spur vertex pair.
            let w = e.map_or(4.0, |eid| g.edges[eid].w);
            dists.push(w / 2.0 + 1.4);
        }
        // The reference's fallback: when per-edge insetting collapses the
        // polygon, retry with the largest distance applied uniformly.
        let inner = match inset_poly(&poly, &dists) {
            Some(p) => p,
            None => {
                let u = dists.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                match inset_poly(&poly, &vec![u; poly.len()]) {
                    Some(p) => p,
                    None => continue,
                }
            }
        };
        // Faces straddling the water stay unbuilt.
        if site.is_water(poly_centroid(&inner)) {
            continue;
        }
        let is_plaza = plaza.is_some_and(|p| point_in_poly(p.center, &poly));
        blocks.push(Block {
            id: format!("blk{}", b_id),
            area: poly_area(&inner).abs(),
            poly: inner,
            face_poly: poly,
            face_ids: ids,
            edge_dists: dists,
            plaza: is_plaza,
        });
        b_id += 1;
    }
    blocks
}

/// `buildParcels` (line 30225) — series platting via bisector edge-cells.
///
/// Each block vertex gets an angle-bisector capped by a ray-cast to the
/// opposite boundary, and each block edge a depth clamp cast from its midpoint,
/// so a lot can never reach past the block's waist. Frontages are granted
/// log-normally around 11 m and then subdivided by the burgage cycle (M-PAR-1),
/// deeper on older streets. Overlapping cells at reflex vertices are dropped by
/// a centroid-in-accepted-lot test, and the survivors trimmed until they
/// conserve the block's area.
///
/// `seed` seeds only the roof-tone stream this port adds; the geometry's
/// randomness is per-block and derived from the block id, exactly as the
/// reference's is.
pub fn build_parcels(
    seed: u32,
    g: &Graph,
    blocks: &[Block],
    anchors_market: Vec2,
    epochs: i32,
    site: &Site,
    rules: Option<&Rules>,
) -> Vec<Parcel> {
    let p = rules.unwrap_or(&DEFAULT_RULES).parcels;
    let mut parcels: Vec<Parcel> = Vec::new();
    let mut pid = 0usize;
    // This port's own stream, kept apart from every geometry draw. See
    // `Parcel::tone`.
    let mut tone_rng: Substream = stream(seed, "roof-tone");

    for blk in blocks {
        if blk.plaza {
            continue;
        }
        let poly = &blk.poly; // CCW
        let n = poly.len();
        if n < 3 {
            continue;
        }
        let mut r_b = stream(fnv1a(&blk.id), &format!("parcels/{}", blk.id));
        let mut cand: Vec<Parcel> = Vec::new();
        let d_m = poly_centroid(poly).dist(anchors_market);
        // M-PAR-2: block-uniform target depth, shallower at the core where
        // land is dear.
        let depth_target = js_max(
            14.0,
            js_min(
                46.0,
                r_b.logn(if d_m < 160.0 { 22.0 } else { 30.0 }, p.plot_depth_variance),
            ),
        );

        // Vertex bisectors, capped by ray-cast to the opposite boundary.
        let mut bis = Vec::with_capacity(n);
        let mut cap = Vec::with_capacity(n);
        for i in 0..n {
            let pt = poly[i];
            let prev = poly[(i + n - 1) % n];
            let next = poly[(i + 1) % n];
            let d1 = (pt - prev).norm();
            let d2 = (next - pt).norm();
            let n1 = Vec2::new(-d1.y, d1.x);
            let n2 = Vec2::new(-d2.y, d2.x);
            let mut b = n1 + n2;
            if b.len() < 1e-6 {
                b = n1;
            }
            b = b.norm();
            let far = pt + b * 2000.0;
            let mut t_min = 1e9;
            for j in 0..n {
                if j == i || j == (i + n - 1) % n {
                    continue;
                }
                if let Some(h) = seg_int(pt, far, poly[j], poly[(j + 1) % n])
                    && h.t > 1e-4
                {
                    t_min = js_min(t_min, h.t * 2000.0);
                }
            }
            bis.push(b);
            cap.push(js_max(2.0, js_min(t_min * 0.42, depth_target * 1.35)));
        }

        // Per-edge depth clamp, cast from the edge midpoint along the inward
        // normal: keeps sum(parcels) <= block area.
        let mut edge_depth = Vec::with_capacity(n);
        for i in 0..n {
            let a = poly[i];
            let b = poly[(i + 1) % n];
            let mid = a.lerp(b, 0.5);
            let d = (b - a).norm();
            let nl = Vec2::new(-d.y, d.x);
            let far = mid + nl * 2000.0;
            let mut t_min = 1e9;
            for j in 0..n {
                if j == i {
                    continue;
                }
                if let Some(h) = seg_int(mid, far, poly[j], poly[(j + 1) % n])
                    && h.t > 1e-4
                {
                    t_min = js_min(t_min, h.t * 2000.0);
                }
            }
            edge_depth.push(js_min(depth_target, t_min * 0.42));
        }

        // Per-edge cells -> strip parcels.
        for i in 0..n {
            let a = poly[i];
            let b = poly[(i + 1) % n];
            let e_len = a.dist(b);
            if e_len < 7.0 || edge_depth[i] < 4.0 {
                continue;
            }
            let la = js_min(cap[i], edge_depth[i]);
            let lb = js_min(cap[(i + 1) % n], edge_depth[i]);
            if la < 4.0 && lb < 4.0 {
                continue;
            }
            let back_a = a + bis[i] * la;
            let back_b = b + bis[(i + 1) % n] * lb;

            // Frontage widths: grant then subdivide (M-PAR-1).
            let eref = g.edge_between(blk.face_ids[i], blk.face_ids[(i + 1) % n]);
            let age = eref.map_or(epochs as f64, |eid| {
                js_max(0.0, (epochs - g.edges[eid].epoch) as f64)
            });
            let mut widths: Vec<f64> = Vec::new();
            let mut acc = 0.0f64;
            while acc < e_len - 3.0 {
                let w = js_max(4.5, js_min(16.0, r_b.logn(11.0, p.frontage_width_variance)));
                let mut parts = vec![w];
                // `Math.min(P.subdivisionCap, Math.floor(age/3))` -- NaN when
                // the rule is NaN, and `s < NaN` is false, so the burgage cycle
                // runs zero times. `ParcelRules::subdivision_cap` is `f64` for
                // exactly this reason; the comparison reproduces it.
                let splits = js_min(p.subdivision_cap, (age / 3.0).floor());
                let mut s = 0.0f64;
                while s < splits {
                    let mut np: Vec<f64> = Vec::with_capacity(parts.len() * 2);
                    for pw in &parts {
                        if *pw > 6.4 && r_b.chance(0.4) {
                            let f = r_b.range(0.4, 0.6);
                            np.push(pw * f);
                            np.push(pw * (1.0 - f));
                        } else {
                            np.push(*pw);
                        }
                    }
                    parts = np;
                    s += 1.0;
                }
                let empty = parts.is_empty();
                for pw in &parts {
                    if acc + pw > e_len - 2.0 {
                        break;
                    }
                    widths.push(*pw);
                    acc += pw;
                }
                if empty || acc + 4.5 > e_len - 2.0 {
                    break;
                }
            }
            if widths.is_empty() {
                continue;
            }

            // Stretch to fill the frontage exactly.
            //
            // The reference writes `(eLen)/(acc||eLen)`, and JS `||` is falsy
            // for **NaN as well as zero** — so a NaN `acc` takes `eLen` there
            // and the scale comes out 1, not NaN. `acc == 0.0` alone would
            // miss that and propagate the NaN into every corner of every lot
            // on this frontage. Reachable: `applyPlotChaos` writes a NaN
            // slider straight into `frontageWidthVariance`, `logn` returns
            // NaN, and `js_min`/`js_max` propagate it by design (which is why
            // they exist). `-0.0` is covered too: `-0.0 == 0.0` is true.
            let denom = if acc == 0.0 || acc.is_nan() { e_len } else { acc };
            let scale = e_len / denom;
            let mut f0 = 0.0f64;
            for w in &widths {
                let f1 = f0 + w * scale / e_len;
                let p0 = a.lerp(b, f0);
                let p1 = a.lerp(b, js_min(1.0, f1));
                let q0 = back_a.lerp(back_b, f0);
                let q1 = back_a.lerp(back_b, js_min(1.0, f1));
                let quad = vec![p0, p1, q1, q0];
                let area = poly_area(&quad).abs();
                // The full footprint, not the centroid: a lot whose centroid is
                // dry but whose back corners dip into the channel would still
                // float a building in the water.
                let margin = if site.kind == "river" {
                    site.river_w / 2.0 + 1.0
                } else {
                    3.0
                };
                let wet = quad
                    .iter()
                    .any(|q| site.is_water(*q) || site.river_dist(*q) < margin);
                // A self-intersecting (bowtie) quad is not a buildable lot at
                // all -- at a reflex vertex the front/back correspondence can
                // flip, putting the area-weighted centroid outside the shape.
                if !wet && (26.0..=2600.0).contains(&area) && !poly_self_intersects(&quad) {
                    cand.push(Parcel {
                        id: format!("par{}", pid),
                        frontage: p0.dist(p1),
                        depth: (p0.dist(q0) + p1.dist(q1)) / 2.0,
                        poly: quad,
                        block: blk.id.clone(),
                        area,
                        age,
                        edge_cls: eref.map_or("street", |eid| g.edges[eid].cls),
                        tone: tone_rng.u(),
                    });
                    pid += 1;
                }
                f0 = f1;
                if f0 >= 1.0 {
                    break;
                }
            }
        }

        // Reflex-vertex cells can overlap: drop any lot whose centroid falls
        // inside an already-accepted one, then trim until the block's area is
        // conserved.
        let mut acc_p: Vec<Parcel> = Vec::new();
        for c in cand {
            let ct = poly_centroid(&c.poly);
            if !acc_p.iter().any(|q| point_in_poly(ct, &q.poly)) {
                acc_p.push(c);
            }
        }
        let mut sum: f64 = acc_p.iter().map(|q| q.area).sum();
        let max_a = poly_area(poly).abs() * 0.97;
        while sum > max_a && !acc_p.is_empty() {
            let q = acc_p
                .pop()
                .expect("non-empty checked by the loop condition");
            sum -= q.area;
        }
        parcels.extend(acc_p);
    }
    parcels
}

#[cfg(test)]
mod tests;

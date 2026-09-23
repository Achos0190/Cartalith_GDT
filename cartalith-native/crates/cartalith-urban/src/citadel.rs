//! A citadel astride the town wall — **not a reference port.** New engine work
//! under Ruling I (`LARGE_ITEM_RULINGS.md`, 2026-09-12), sited by Ruling AC
//! (2026-09-23: *"largest settlements only, by size tier — not gated on
//! faction-seat status"*), reasoned against the owner's target plan
//! `design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`.
//!
//! The reference (v2.11) has nothing like it under any name: [`crate::fortify`]
//! builds the curtain, gates, spurs and the bastioned star fort, and the only
//! castle in this crate is the `"keep"` civic style in
//! [`crate::amenities::build_civic`] — a hall on the **market square**, drawn
//! from the plaza's own frame and the `"civic"` substream. It is not reused
//! here: it cannot be moved off the plaza without moving every civic hall, and
//! the citadel's building has to sit against the citadel's outer wall, not face
//! a square. Its *vocabulary* is kept — a rank-scaled rectangle — and nothing
//! else.
//!
//! # What the owner's plan shows, and what is built
//!
//! On the plan the citadel is a walled quadrilateral set **astride** the curtain
//! on the north-east: most of it projects outside the town, its inner face runs
//! just inside the circuit, towers stand at its corners and where its walls
//! meet the curtain, one long building lines its outer side, the rest is an
//! open court, and a street from inside the town reaches it. [`build_citadel`]
//! builds exactly that, as a rectangle oriented on the curtain's own tangent:
//!
//! - **`wall`** — four corners; the inner face [`INSET_FRAC`] of the depth
//!   inside the curtain, the rest outside, so the curtain crosses it.
//! - **`towers`** — the four corners plus every point where the citadel's walls
//!   cross the town's circuit.
//! - **`keep`** — one large building against the outer face.
//! - **`court`** — the open ground between the keep and the inner face.
//! - **`gate`** — the internal gate, mid-inner-face, **inside** the town, and
//!   **`approach`** — a street from it to the nearest live junction inside the
//!   circuit, added to the graph.
//!
//! # Where it goes
//!
//! On the **highest ground** the drawn land arc offers (`site.height` at the
//! citadel's centre; every site has one — the real heightfield, or the
//! synthetic proxy's rise and hills), which is where a castle was put: it
//! commands the town. Every candidate must be on dry land at its corners,
//! edge midpoints and centre, keep a clear [`GATE_CLEARANCE`] beyond its half
//! width from every town gate, cross no live primary road, and have a
//! reachable junction for its approach. No candidate passes → no citadel,
//! which is a real state, not an error.
//!
//! # Which towns get one — Ruling AC's size tier
//!
//! [`CITADEL_MIN_POP`] (10 000) on `pop_target`, on the organic plan, on a
//! circuit that is not a bastioned trace. The same *shape* of gate as the star
//! fort's (`generate.rs`: `pop_target >= FORT_MIN && walls && organic`), and a
//! tier boundary this crate already draws rather than a new one: 10 000 is
//! where [`crate::amenities::build_civic`] names the hall a "Town hall" rather
//! than a "Guildhall" and where [`crate::wallside`] runs its third faubourg
//! run. `cartalith-civ`'s base populations put a Capital (15 000) and a
//! Metropolis above it and a City (6 000) below, so in practice this is the
//! largest settlements — by *size*, as the ruling says: a small capital does
//! not get one, a grown city does.
//!
//! **A bastioned town gets none.** Its trace already is the state's
//! fortification, and a citadel on an artillery trace would itself be a
//! bastioned work (a pentagonal citadel), which is a different build. Named
//! here rather than hidden: ponytail — add when the owner wants one there.
//!
//! # The open question, defaulted: the citadel is NOT inside the circuit
//!
//! Ruling AC left open *"whether the citadel's area counts inside the wall
//! circuit for growth purposes"*. **Default: it does not.** `wall_state.ring`
//! is read and never written here (this function takes it by `&`), so the
//! circuit's area, `building_intramural`, `clear_fort_zone`'s band and every
//! containment test are exactly what they were. The citadel is its own
//! enclosure on [`crate::Town::citadel`]. It is also built after `grow`, so the
//! epoch loop's expansion trigger never sees it either way. The conservative
//! choice: nothing about the town's growth area silently changes. Revisit if
//! the owner rules the other way.
//!
//! # Golden consequence
//!
//! Draws come from its own `"citadel"` substream, and the stage runs after every
//! other lot and building exists, so a town that does not qualify is untouched
//! byte for byte. A qualifying one loses the buildings, lots, details and
//! non-primary streets under the citadel and gains the approach street —
//! disclosed case by case in `generate/tests/golden.rs`'s header.

use crate::geom::{Vec2, dist_pt_seg, point_in_poly, poly_centroid, seg_int};
use crate::graph::Graph;
use crate::growth::{WallState, ring_crossings};
use crate::rng::stream;
use crate::site::Site;

#[cfg(test)]
mod tests;

/// Ruling AC's size tier: the smallest `pop_target` that gets a citadel. See
/// the module header for why 10 000 is a boundary this crate already draws.
pub const CITADEL_MIN_POP: f64 = 10000.0;
/// Width along the curtain, metres — drawn once from `"citadel"`. Scaled off
/// the owner's plan, where the citadel spans roughly a quarter of a town
/// ~450 m across.
pub const WIDTH: (f64, f64) = (90.0, 120.0);
/// Depth across the curtain, metres — the second and last draw.
pub const DEPTH: (f64, f64) = (80.0, 100.0);
/// How much of the depth lies inside the curtain. The plan's citadel projects
/// almost wholly outward with its inner face just inside the circuit.
pub const INSET_FRAC: f64 = 0.2;
/// Clearance beyond the half width to every town gate, metres — the citadel
/// never sits on a gate road.
pub const GATE_CLEARANCE: f64 = 40.0;
/// Farthest the approach street may run to its junction, metres.
pub const APPROACH_MAX: f64 = 120.0;
/// Tower radius, metres.
pub const TOWER_R: f64 = 6.0;
/// Wall thickness the keep and the court stand clear of, metres.
pub const WALL_T: f64 = 6.0;

/// Provenance for the enclosure.
pub const PROV: &str = "Citadel: the lord's or garrison's own walled enclosure set astride the \
    town curtain, commanding the highest ground — towers at its corners and where it meets the \
    town wall, a great hall against its outer side and an open court, entered by a gate from \
    inside the town so it could hold against the townsfolk as well as an enemy (Ruling I; not a \
    reference port).";
/// Provenance for the approach street.
pub const APPROACH_PROV: &str = "Castle street: the one road from the town to the citadel's \
    inner gate (Ruling I).";

/// One citadel. Every polygon is in the town's own metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Citadel {
    /// Four corners: inner-left, inner-right, outer-right, outer-left.
    pub wall: Vec<Vec2>,
    /// The corners, then every crossing of `wall` with the town circuit.
    pub towers: Vec<Vec2>,
    pub tower_r: f64,
    /// The one large building, against the outer face.
    pub keep: Vec<Vec2>,
    /// The open court, between the keep and the inner face.
    pub court: Vec<Vec2>,
    /// The internal gate — mid inner face, inside the town circuit.
    pub gate: Vec2,
    /// `[gate, junction]` — the street added to the graph.
    pub approach: Vec<Vec2>,
    /// The point on the drawn curtain the citadel is centred across.
    pub on_curtain: Vec2,
    pub prov: &'static str,
}

/// A rectangle `w` along `t` by `d` along `n`, centred on `c`, wound
/// inner-left, inner-right, outer-right, outer-left (`n` points outward).
fn rect(c: Vec2, t: Vec2, n: Vec2, w: f64, d: f64) -> Vec<Vec2> {
    let (hw, hd) = (w / 2.0, d / 2.0);
    vec![c - t * hw - n * hd, c + t * hw - n * hd, c + t * hw + n * hd, c - t * hw + n * hd]
}

fn crosses_rect(a: Vec2, b: Vec2, r: &[Vec2]) -> bool {
    point_in_poly(a, r)
        || point_in_poly(b, r)
        || (0..r.len()).any(|i| seg_int(a, b, r[i], r[(i + 1) % r.len()]).is_some())
}

/// Build the citadel for a qualifying town, or [`None`]. Mutates `g` only when
/// it returns `Some`: the non-primary streets under the footprint are killed and
/// the approach street is added. The caller gates on the size tier; this
/// function refuses on its own for no circuit or a bastioned one.
pub fn build_citadel(seed: u32, site: &Site, wall: &WallState, g: &mut Graph) -> Option<Citadel> {
    let (Some(ring), Some(arc)) = (wall.ring.as_deref(), wall.land_arc.as_deref()) else {
        return None;
    };
    if wall.style == "bastioned" || arc.len() < 3 || ring.len() < 3 {
        return None;
    }
    let mut r = stream(seed, "citadel");
    let w = r.range(WIDTH.0, WIDTH.1);
    let d = r.range(DEPTH.0, DEPTH.1);
    let centre = wall.centroid.unwrap_or_else(|| poly_centroid(ring));
    let closed = wall.water_closure.is_none();
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let dry = |p: Vec2| {
        p.x > 20.0
            && p.y > 20.0
            && p.x < site.wm - 20.0
            && p.y < site.hm - 20.0
            && !site.is_water(p)
            && !(rk && site.river_dist(p) < site.river_w / 2.0 + 12.0)
    };
    let live_primaries: Vec<(Vec2, Vec2)> = g
        .edges
        .iter()
        .filter(|e| e.alive && e.cls == "primary")
        .map(|e| (g.nodes[e.a].pt(), g.nodes[e.b].pt()))
        .collect();

    let n_arc = arc.len();
    let mut best: Option<(f64, Vec<Vec2>, Vec2, Vec2, Vec2, usize)> = None;
    for i in 0..n_arc {
        let p = arc[i];
        let (prev, next) = if closed {
            (arc[(i + n_arc - 1) % n_arc], arc[(i + 1) % n_arc])
        } else {
            (arc[i.saturating_sub(1)], arc[(i + 1).min(n_arc - 1)])
        };
        let t = (next - prev).norm();
        let mut n = t.rot90();
        if n.dot(p - centre) < 0.0 {
            n = n * -1.0;
        }
        let c = p + n * (d / 2.0 - d * INSET_FRAC);
        let wr = rect(c, t, n, w, d);
        let samples = (0..4).flat_map(|k| [wr[k], wr[k].lerp(wr[(k + 1) % 4], 0.5)]);
        if !dry(c) || !samples.into_iter().all(dry) {
            continue;
        }
        if wall.gates.iter().any(|gt| gt.pt.dist(p) < w / 2.0 + GATE_CLEARANCE) {
            continue;
        }
        if live_primaries.iter().any(|&(a, b)| crosses_rect(a, b, &wr)) {
            continue;
        }
        // Astride, strictly: the inner face (and its gate) inside the town, the
        // outer corners outside. A tight bend of the curtain fails this.
        let gate = wr[0].lerp(wr[1], 0.5);
        if !point_in_poly(gate, ring)
            || !point_in_poly(wr[0], ring)
            || !point_in_poly(wr[1], ring)
            || point_in_poly(wr[2], ring)
            || point_in_poly(wr[3], ring)
        {
            continue;
        }
        // The approach: the nearest live junction inside the circuit, outside
        // the citadel, reachable without crossing either wall.
        let mut junction: Option<(f64, usize)> = None;
        for node in &g.nodes {
            if g.live_degree(node.id) == 0 {
                continue;
            }
            let q = node.pt();
            let dq = q.dist(gate);
            if !(4.0..=APPROACH_MAX).contains(&dq) || junction.is_some_and(|(bd, _)| dq >= bd) {
                continue;
            }
            if !point_in_poly(q, ring)
                || point_in_poly(q, &wr)
                || point_in_poly(gate.lerp(q, 0.5), &wr)
                || !ring_crossings(ring, gate, q).is_empty()
            {
                continue;
            }
            // It must keep a street after the footprint is cleared, or the
            // approach would lead to an island.
            let keeps_a_street = g.live_adj(node.id).any(|eid| {
                let e = &g.edges[eid];
                let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
                !point_in_poly(a, &wr) && !point_in_poly(b, &wr) && !point_in_poly(a.lerp(b, 0.5), &wr)
            });
            if !keeps_a_street {
                continue;
            }
            junction = Some((dq, node.id));
        }
        let Some((_, jid)) = junction else { continue };
        let h = site.height(c);
        if best.as_ref().is_none_or(|b| h > b.0) {
            best = Some((h, wr, p, t, n, jid));
        }
    }
    let (_, wr, p, t, n, jid) = best?;

    // Towers: the corners, then where the citadel's walls cross the circuit.
    let mut towers = wr.clone();
    for k in 0..4 {
        for x in ring_crossings(ring, wr[k], wr[(k + 1) % 4]) {
            // Only an exact repeat is dropped: a junction a few metres from a
            // corner is still where the two walls meet.
            if !towers.iter().any(|q| q.dist(x) < 1e-6) {
                towers.push(x);
            }
        }
    }
    // The keep against the outer face; the court between it and the inner face.
    let kw = w * 0.6;
    let kd = d * 0.28;
    let outer_mid = wr[2].lerp(wr[3], 0.5);
    let keep = rect(outer_mid - n * (WALL_T + kd / 2.0), t, n, kw, kd);
    let inner_mid = wr[0].lerp(wr[1], 0.5);
    let court_d = d - 2.0 * WALL_T - kd;
    let court = rect(inner_mid + n * (WALL_T + court_d / 2.0), t, n, w - 2.0 * WALL_T, court_d);

    // The ground under it stops being street: every non-primary edge touching
    // the footprint is killed (the candidate test already refused primaries).
    for eid in 0..g.edges.len() {
        let e = &g.edges[eid];
        if !e.alive {
            continue;
        }
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        if point_in_poly(a, &wr) || point_in_poly(b, &wr) || point_in_poly(a.lerp(b, 0.5), &wr) {
            crate::cleanup::kill_edge(g, eid);
        }
    }
    let jpt = g.nodes[jid].pt();
    g.add_street(inner_mid.x, inner_mid.y, jpt.x, jpt.y, "street", 5.0, 0, APPROACH_PROV);

    Some(Citadel {
        wall: wr,
        towers,
        tower_r: TOWER_R,
        keep,
        court,
        gate: inner_mid,
        approach: vec![inner_mid, jpt],
        on_curtain: p,
        prov: PROV,
    })
}

/// What sits under a citadel, as indices — reported, not applied, the same
/// report-and-apply shape as [`crate::cleanup::clear_fort_zone`]. Removal lists
/// are **descending**, so a caller can apply them in the order they arrive.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CitadelSweep {
    pub buildings_removed: Vec<usize>,
    pub parcels_cleared: Vec<usize>,
    pub details_removed: Vec<usize>,
}

/// A building goes if any vertex is inside the citadel or it is within 3 m of
/// the approach street's centreline; a lot is cleared on its centroid; a detail
/// goes on its anchor.
pub fn citadel_sweep(
    c: &Citadel,
    building_polys: &[Vec<Vec2>],
    parcel_polys: &[Vec<Vec2>],
    detail_pts: &[Option<Vec2>],
) -> CitadelSweep {
    let (a, b) = (c.approach[0], c.approach[1]);
    let on_approach = |poly: &[Vec2]| {
        poly.iter().any(|q| dist_pt_seg(*q, a, b) < 3.0)
            || (0..poly.len()).any(|i| seg_int(a, b, poly[i], poly[(i + 1) % poly.len()]).is_some())
    };
    let under = |poly: &[Vec2]| poly.iter().any(|q| point_in_poly(*q, &c.wall));
    CitadelSweep {
        buildings_removed: (0..building_polys.len())
            .rev()
            .filter(|&i| under(&building_polys[i]) || on_approach(&building_polys[i]))
            .collect(),
        parcels_cleared: (0..parcel_polys.len())
            .filter(|&i| point_in_poly(poly_centroid(&parcel_polys[i]), &c.wall))
            .collect(),
        details_removed: (0..detail_pts.len())
            .rev()
            .filter(|&i| detail_pts[i].is_some_and(|p| point_in_poly(p, &c.wall)))
            .collect(),
    }
}

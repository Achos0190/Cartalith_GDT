//! Owner finding F27 / Ruling GL (2026-11-22): **a settlement is founded where roads converge.**
//!
//! > *"When roads converge often a settlement/waypoint arises/is founded. The chance is even higher
//! > when this happens with a river in its proximity and in between larger settlements."*
//!
//! One pure pass, [`civ_junction_settlements`], over the drawn `ways` and the settlements that
//! exist. It serves both halves of the ruling: world generation (after the roads and the village
//! pass) and the collapse/recovery timeline (each recovery step, over the roads carried forward).
//! It is **not** a port: the reference has nothing like it. Behind `civ.junction_settlements`
//! (section 7p: on in the app, off in `WorldParams::defaults`, off for a save without the key).
//!
//! * **Junction**: the end of a way (consolidation cuts a way where a trunk branches) that lies
//!   within [`JUNCTION_TOL_CELLS`] of enough other ways for a degree of 3 (an end counts 1, a way
//!   passing through 2, so a T is a junction) and further than that from every
//!   settlement (a junction at a settlement is already a settlement). Degree 3 or more.
//! * **Chance**: [`JUNCTION_BASE_CHANCE`], raised by [`JUNCTION_RIVER_BOOST`] when a river cell lies
//!   within [`JUNCTION_RIVER_REACH_KM`], and by up to [`JUNCTION_MIDWAY_BOOST`] by how evenly the
//!   site sits between its two nearest larger settlements (Town or above, both within
//!   [`JUNCTION_LARGE_REACH_KM`]). All three numbers are labelled judgements (owner gave no figure);
//!   they are measured in `docs/evidence/junctions-2026-11-22/`.
//! * **Spacing**: the village spacing ([`VILLAGE_SPACING_KM`]) from every settlement and every
//!   settlement this pass adds. A violation is a bug, never a tolerated rate.
//!
//! At generation it runs before the village pass (see `compute_civilisation`). Must never run on the keep path (SG-02): it authors settlements.

use crate::{name_style, suppression_radius_cells, NamedSettlement, SettlementKind, VillageSettlement, Way, CIV_VILLAGE_CAP, VILLAGE_SPACING_KM};
use std::collections::HashMap;

/// A way end within this many cells of another way counts as meeting it (labelled judgement: the
/// consolidated ways' own endpoint snap is about one cell).
pub const JUNCTION_TOL_CELLS: f64 = 2.5;
/// Founding chance at a bare junction (labelled judgement).
pub const JUNCTION_BASE_CHANCE: f64 = 0.35;
/// Added multiplier when a river is near: chance x (1 + this) (labelled judgement; "even higher").
pub const JUNCTION_RIVER_BOOST: f64 = 1.0;
/// Added multiplier at a perfectly midway site: chance x (1 + this x midness) (labelled judgement).
pub const JUNCTION_MIDWAY_BOOST: f64 = 0.8;
/// "Near a river" (labelled judgement).
pub const JUNCTION_RIVER_REACH_KM: f64 = 4.0;
/// Both larger neighbours must be this near for the site to count as "in between" (judgement).
pub const JUNCTION_LARGE_REACH_KM: f64 = 150.0;
/// The generation stream's seed, xor the settlement count (a labelled constant: arbitrary).
pub const JUNCTION_STREAM_SEED: u32 = 0x4A55_4E43;
/// Hard ceiling on a founding chance.
pub const JUNCTION_MAX_CHANCE: f64 = 0.95;

/// What the pass needs to know about the world, as predicates so generation and the timeline can
/// each supply theirs (generation's dry test also reads lakes; the timeline's reads the field).
pub struct JunctionWorld<'a> {
    pub gw: usize,
    pub gh: usize,
    pub map_width_km: f64,
    /// Dry land a hamlet may stand on.
    pub dry: &'a dyn Fn(usize, usize) -> bool,
    /// A river channel cell.
    pub river: &'a dyn Fn(usize, usize) -> bool,
    pub cultures: &'a [&'a str],
}

/// One junction with the two things that scale its chance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Junction {
    pub x: f64,
    pub y: f64,
    /// Way incidences here: ends count 1, a way passing through 2 (3 or more).
    pub degree: usize,
}

/// Every junction of `ways` that is not at a settlement. Pure and deterministic (way order).
pub fn civ_junctions(ways: &[Way], places: &[NamedSettlement]) -> Vec<Junction> {
    let tol = JUNCTION_TOL_CELLS;
    // Bucket every way densified to <= 1 cell steps, so "near another way" is a point test.
    // Each sample carries whether it is within `tol` of its way's own end: a way merely ending near
    // `p` is one incidence there, a way passing through it is two (a T of two ways is degree 3).
    let mut grid: HashMap<(i32, i32), Vec<(usize, f64, f64, bool)>> = HashMap::new();
    let mut put = |wi: usize, x: f64, y: f64, near_end: bool| grid.entry(((x / tol).floor() as i32, (y / tol).floor() as i32)).or_default().push((wi, x, y, near_end));
    for (wi, w) in ways.iter().enumerate().filter(|(_, w)| !w.hidden) {
        let (first, last) = (w.pts.first().copied().unwrap_or((0.0, 0.0)), w.pts.last().copied().unwrap_or((0.0, 0.0)));
        let ends = |x: f64, y: f64| (x - first.0).hypot(y - first.1) <= tol || (x - last.0).hypot(y - last.1) <= tol;
        for pair in w.pts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let n = ((b.0 - a.0).hypot(b.1 - a.1).ceil() as usize).max(1);
            for k in 0..n {
                let t = k as f64 / n as f64;
                let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                put(wi, x, y, ends(x, y));
            }
        }
        if let Some(&(x, y)) = w.pts.last() {
            put(wi, x, y, true);
        }
    }
    let mut out = Vec::new();
    for (wi, w) in ways.iter().enumerate().filter(|(_, w)| !w.hidden) {
        for p in [w.pts.first(), w.pts.last()].into_iter().flatten() {
            if places.iter().any(|s| (s.placement.x as f64 - p.0).hypot(s.placement.y as f64 - p.1) <= tol) {
                continue;
            }
            let (bx, by) = ((p.0 / tol).floor() as i32, (p.1 / tol).floor() as i32);
            // (way, incidences): 1 if it only ends here, 2 if it passes through.
            let mut others: Vec<(usize, usize)> = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    for &(wj, x, y, near_end) in grid.get(&(bx + dx, by + dy)).into_iter().flatten() {
                        if wj == wi || (x - p.0).hypot(y - p.1) > tol {
                            continue;
                        }
                        match others.iter_mut().find(|o| o.0 == wj) {
                            Some(o) => o.1 = o.1.max(if near_end { 1 } else { 2 }),
                            None => others.push((wj, if near_end { 1 } else { 2 })),
                        }
                    }
                }
            }
            let degree = 1 + others.iter().map(|o| o.1).sum::<usize>();
            if degree >= 3 {
                out.push(Junction { x: p.0, y: p.1, degree });
            }
        }
    }
    out
}

/// How evenly `(x, y)` sits between its two nearest larger settlements: 1 = equidistant, 0 = one
/// far closer, or fewer than two within [`JUNCTION_LARGE_REACH_KM`].
pub fn midness(x: f64, y: f64, places: &[NamedSettlement], cell_km: f64) -> f64 {
    let mut d: Vec<f64> = places
        .iter()
        .filter(|s| matches!(s.placement.kind, SettlementKind::Metropolis | SettlementKind::Capital | SettlementKind::City | SettlementKind::Town))
        .map(|s| (s.placement.x as f64 - x).hypot(s.placement.y as f64 - y) * cell_km)
        .collect();
    d.sort_by(|a, b| a.total_cmp(b));
    match d.as_slice() {
        [a, b, ..] if *b <= JUNCTION_LARGE_REACH_KM => 1.0 - (b - a) / (a + b),
        _ => 0.0,
    }
}

/// The founding chance at a junction (see the module doc). `near_river` and `mid` as above.
pub fn founding_chance(near_river: bool, mid: f64, scale: f64) -> f64 {
    let c = JUNCTION_BASE_CHANCE * (1.0 + if near_river { JUNCTION_RIVER_BOOST } else { 0.0 }) * (1.0 + JUNCTION_MIDWAY_BOOST * mid);
    (c * scale).min(JUNCTION_MAX_CHANCE)
}

/// Found hamlets at `ways`' junctions. `scale` multiplies the chance (1 at generation; a timeline
/// step passes its share of a century). `rng` is the caller's own stream: one `next_f64` per
/// candidate rolled, then the name draws, in a fixed candidate order (river first, then most
/// midway, then way order), so the result is a function of the inputs alone.
pub fn civ_junction_settlements(
    ways: &[Way],
    places: &[NamedSettlement],
    w: &JunctionWorld,
    scale: f64,
    rng: &mut cartalith_rng::Mulberry32,
) -> Vec<VillageSettlement> {
    let cell_km = w.map_width_km / w.gw as f64;
    let spacing = suppression_radius_cells(VILLAGE_SPACING_KM, w.gw, w.map_width_km);
    let reach = (JUNCTION_RIVER_REACH_KM / cell_km).ceil().max(2.0) as i64;
    let near_river = |x: usize, y: usize| -> bool {
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < w.gw && (ny as usize) < w.gh && (w.river)(nx as usize, ny as usize) {
                    return true;
                }
            }
        }
        false
    };
    let mut cands: Vec<(bool, f64, usize, usize, usize)> = civ_junctions(ways, places)
        .into_iter()
        .enumerate()
        .filter_map(|(i, j)| {
            let (x, y) = ((j.x.round().max(0.0) as usize).min(w.gw - 1), (j.y.round().max(0.0) as usize).min(w.gh - 1));
            (w.dry)(x, y).then(|| (near_river(x, y), midness(x as f64, y as f64, places, cell_km), i, x, y))
        })
        .collect();
    cands.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2)));
    let mut added: Vec<VillageSettlement> = Vec::new();
    for (river, mid, _, x, y) in cands {
        if added.len() >= CIV_VILLAGE_CAP {
            break;
        }
        let far = |px: f64, py: f64| (px - x as f64).powi(2) + (py - y as f64).powi(2) >= spacing * spacing;
        if !places.iter().all(|s| far(s.placement.x as f64, s.placement.y as f64)) || !added.iter().all(|s| far(s.x as f64, s.y as f64)) {
            continue;
        }
        if rng.next_f64() >= founding_chance(river, mid, scale) {
            continue;
        }
        let faction = places
            .iter()
            .min_by(|a, b| {
                let d = |s: &NamedSettlement| (s.placement.x as f64 - x as f64).hypot(s.placement.y as f64 - y as f64);
                d(a).total_cmp(&d(b))
            })
            .map_or(1, |s| s.placement.faction);
        let name = name_style::settle_name_for_key(rng, w.cultures, faction);
        added.push(VillageSettlement { x, y, name, faction });
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SettlementPlacement, WayType};

    fn way(pts: Vec<(f64, f64)>) -> Way {
        Way { tid: 0, pts, brks: vec![], km: 0.0, name: String::new(), way_type: WayType::Road, a_idx: 0, b_idx: 0, hidden: false }
    }
    fn town(x: usize, y: usize, kind: SettlementKind) -> NamedSettlement {
        NamedSettlement { tid: 0, placement: SettlementPlacement { x, y, suit: 0.0, faction: 1, capital: false, kind, coastal: false }, name: "T".into(), pop: 1 }
    }

    /// Protects: a Y of three ways ending at one bare point is one junction of degree 3; a plain
    /// bend (two ways meeting) and a way ending at a settlement are not.
    #[test]
    fn three_ways_meeting_away_from_a_settlement_is_a_junction() {
        let ways = vec![way(vec![(0.0, 50.0), (50.0, 50.0)]), way(vec![(50.0, 50.0), (100.0, 20.0)]), way(vec![(50.0, 50.0), (100.0, 80.0)])];
        let js = civ_junctions(&ways, &[town(0, 50, SettlementKind::Town)]);
        assert!(!js.is_empty() && js.iter().all(|j| j.degree == 3 && (j.x - 50.0).abs() < 1.0));
        assert!(civ_junctions(&ways[..2], &[]).is_empty(), "two ways meeting is a bend");
        // Three ends 2 cells apart still meet (tolerance 2.5), 3 cells apart do not.
        let near = |d: f64| vec![way(vec![(0.0, 50.0), (50.0, 50.0)]), way(vec![(50.0 + d, 50.0), (100.0, 20.0)]), way(vec![(50.0, 50.0 + d), (100.0, 80.0)])];
        assert!(!civ_junctions(&near(2.0), &[]).is_empty() && civ_junctions(&near(4.0), &[]).is_empty());
        let t = vec![way(vec![(0.0, 50.0), (100.0, 50.0)]), way(vec![(50.0, 50.0), (50.0, 100.0)])];
        assert!(civ_junctions(&t, &[]).iter().any(|j| j.degree == 3 && (j.x - 50.0).abs() < 1.0), "a way ending on another's interior is a T");
        assert!(civ_junctions(&ways, &[town(50, 50, SettlementKind::Town)]).is_empty(), "a junction at a settlement is that settlement");
    }

    /// Protects: the ruling's two boosts, as literals. Midway beats lopsided; river beats dry.
    #[test]
    fn river_and_midway_raise_the_chance() {
        let ps = [town(0, 0, SettlementKind::Town), town(100, 0, SettlementKind::City)];
        assert!((midness(50.0, 0.0, &ps, 1.0) - 1.0).abs() < 1e-9);
        assert!((midness(10.0, 0.0, &ps, 1.0) - (1.0 - 80.0 / 100.0)).abs() < 1e-9);
        assert_eq!(midness(50.0, 0.0, &ps[..1], 1.0), 0.0, "one larger neighbour is not 'in between'");
        assert!((founding_chance(false, 0.0, 1.0) - 0.35).abs() < 1e-12);
        assert!((founding_chance(true, 0.0, 1.0) - 0.70).abs() < 1e-12);
        assert!((founding_chance(false, 1.0, 1.0) - 0.35 * 1.8).abs() < 1e-12, "midway alone: 0.35 x 1.8");
        assert_eq!(founding_chance(true, 1.0, 0.0), 0.0, "scale 0 never founds");
        let far = [town(0, 0, SettlementKind::Town), town(400, 0, SettlementKind::City)];
        assert_eq!(midness(200.0, 0.0, &far, 1.0), 0.0, "a second larger neighbour 200 km off is not 'in between'");
        assert!((founding_chance(true, 1.0, 1.0) - 0.95).abs() < 1e-12, "0.35 x 2 x 1.8 = 1.26, capped");
    }

    /// Protects: spacing (never closer than the village spacing to a settlement or a new hamlet),
    /// dry land, and determinism for one seed.
    #[test]
    fn founding_respects_spacing_land_and_is_deterministic() {
        let ways = vec![
            way(vec![(0.0, 100.0), (100.0, 100.0)]),
            way(vec![(100.0, 100.0), (200.0, 40.0)]),
            way(vec![(100.0, 100.0), (200.0, 160.0)]),
        ];
        let places = vec![town(0, 100, SettlementKind::Town), town(200, 40, SettlementKind::Town)];
        let all_dry = |_: usize, _: usize| true;
        let river = |x: usize, _: usize| x == 100;
        let w = JunctionWorld { gw: 256, gh: 256, map_width_km: 256.0, dry: &all_dry, river: &river, cultures: &[] };
        let run = || civ_junction_settlements(&ways, &places, &w, 100.0, &mut cartalith_rng::Mulberry32::new(7));
        let a = run();
        assert_eq!(a, run());
        assert_eq!(a.len(), 1, "chance forced to the cap by scale; three nearby ends are one site");
        assert!((a[0].x as i64 - 100).abs() <= 3);
        assert!(civ_junction_settlements(&ways, &places, &w, 0.0, &mut cartalith_rng::Mulberry32::new(7)).is_empty(), "chance 0 founds nothing");
        let wet = |_: usize, _: usize| false;
        let w2 = JunctionWorld { dry: &wet, ..w };
        assert!(civ_junction_settlements(&ways, &places, &w2, 100.0, &mut cartalith_rng::Mulberry32::new(7)).is_empty());
    }
}

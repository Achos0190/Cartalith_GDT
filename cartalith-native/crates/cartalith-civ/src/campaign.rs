//! Ruling AW (`LARGE_ITEM_RULINGS.md`, 2026-09-24) -- war campaigns over time,
//! read for one year of the Timeline cursor. `MILITARY_MANPOWER_SCOPE.md` §5
//! is the scope; this module is its three derived readings and nothing else:
//!
//! - **siege rings** -- one per `Siege`-kind conflict active that year, a
//!   circle of [`siege_line_radius_cells`] around the siege's drawn point;
//! - **the front** -- the 4-neighbour cell edges where one of the conflict's
//!   sides meets another in the territory recorded for that year;
//! - **changed hands** -- the cells whose owner moved from one of the
//!   conflict's sides to another between the conflict's start and that year.
//!
//! **Nothing here moves a unit or decides a battle** (the ruling's own last
//! paragraph). Every reading is either what the author drew (a siege's place
//! and years, a conflict's sides) or what the timeline recorded (territory).
//!
//! ## Which territory "that year" means
//!
//! The recorded snapshot **in force** at the year: the latest recorded year at
//! or before it ([`year_in_force`]). An unrecorded year between two recorded
//! ones reads the earlier -- claims hold until the next record says otherwise,
//! which is Ruling AT's base model. This is deliberately not the live claim
//! grid's "holds at" year (`CivData::territory_year`): that one depends on the
//! path the cursor took (scrub back from a later record and the grid keeps the
//! later claims), and a history layer that drew a different front for the same
//! year depending on how you got there would be describing the UI, not the war.
//!
//! No-value discipline (`MISTAKES.md`): a reading that cannot be made is
//! `None`, never an empty list that reads as "no front" or "nothing changed".

use std::collections::BTreeMap;

use crate::conflict::{Conflict, ConflictAnchor, ConflictKind};
use crate::timeline::{civ_territory_at, TimelineSnapshot};

/// One Roman mile (*mille passus*, 1 000 double paces), in km. The customary
/// modern value, 1.48 km; it is the unit Caesar measured Alesia's works in.
pub const ROMAN_MILE_KM: f64 = 1.48;

/// The circuit of the siege line drawn around a besieged place, in Roman miles:
/// Alesia's contravallation, the inner ring that faced the town -- "eleven
/// miles" (*XI milia passuum*, Caesar, *De Bello Gallico* VII.69). One attested
/// siege work, used as the stated scale of "a siege line", not a fit to any
/// data: `MILITARY_MANPOWER_SCOPE.md` §5.1 says why no army size scales it.
pub const SIEGE_LINE_CIRCUIT_MILES: f64 = 11.0;

/// The smallest ring radius, in cells. A cell's circumradius is `sqrt(2)/2`
/// (0.707) cells, so a ring any tighter would cut through the besieged cell it
/// is meant to surround; 1.0 is the smallest whole-cell radius that clears it.
/// Reached on a coarse map (above ~2.6 km per cell), where the attested radius
/// is smaller than one cell.
pub const SIEGE_RING_MIN_CELLS: f64 = 1.0;

/// The siege line's radius in cells at `km_per_cell`: the circuit's radius
/// (`circuit / 2 pi`, 2.591 km) over the cell size, floored at
/// [`SIEGE_RING_MIN_CELLS`]. `None` when the map has no usable scale -- the
/// ring's size is unknown then, not a default.
pub fn siege_line_radius_cells(km_per_cell: f64) -> Option<f64> {
    if !(km_per_cell.is_finite() && km_per_cell > 0.0) {
        return None;
    }
    let radius_km = SIEGE_LINE_CIRCUIT_MILES * ROMAN_MILE_KM / std::f64::consts::TAU;
    Some((radius_km / km_per_cell).max(SIEGE_RING_MIN_CELLS))
}

/// The latest recorded year at or before `year`, or `None` when the timeline
/// records nothing that early.
pub fn year_in_force(timeline: &[TimelineSnapshot], year: i64) -> Option<i64> {
    timeline.iter().map(|s| s.year).filter(|&y| y <= year).max()
}

/// One siege, read for the year.
#[derive(Debug, Clone, PartialEq)]
pub struct SiegeRing {
    /// The siege's drawn point, resolved against its anchor (grid cells,
    /// continuous: cell `(x, y)` spans `[x, x+1) x [y, y+1)`).
    pub centre: (f64, f64),
    /// [`siege_line_radius_cells`]; `None` when the map has no scale.
    pub radius_cells: Option<f64>,
    /// The settlement the siege is anchored to, when it is anchored to one.
    /// A free-standing siege mark names no place: `None`, not a guess at the
    /// nearest town.
    pub besieged_tid: Option<u64>,
    /// The owner of the cell under `centre` in the territory in force, when
    /// that owner is one of the conflict's sides. `None` when no territory is
    /// recorded, the point is off the grid, or the owner is not a side.
    pub defender: Option<i32>,
    /// The other sides, when the defender is known; `None` otherwise.
    pub besiegers: Option<Vec<i32>>,
}

/// A cell that changed hands between two of the conflict's sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangedCell {
    pub cell: u32,
    pub from: i32,
    pub to: i32,
}

/// One conflict's campaign reading for one year.
#[derive(Debug, Clone, PartialEq)]
pub struct CampaignReading {
    pub conflict_id: u64,
    pub sides: Vec<i32>,
    /// The recorded year the front and the changes are read at
    /// ([`year_in_force`] of the cursor); `None` = nothing recorded that early.
    pub territory_year: Option<i64>,
    /// The recorded year in force at the conflict's start -- the baseline the
    /// changes are measured from; `None` = nothing recorded that early.
    pub baseline_year: Option<i64>,
    /// `Some` only for a `Siege`-kind conflict.
    pub siege: Option<SiegeRing>,
    /// The front: `(a, b)` cell-index pairs, `a < b`, 4-neighbours, owned by
    /// two different sides. Ascending. `None` when the territory in force
    /// cannot be read on this grid.
    pub front_edges: Option<Vec<(u32, u32)>>,
    /// Every cell on either side of a front edge, ascending, unique.
    pub front_cells: Option<Vec<u32>>,
    /// Ascending by cell. `None` when either end (baseline or territory in
    /// force) cannot be read.
    pub changed: Option<Vec<ChangedCell>>,
}

/// Every conflict active in `year`, read ([`CampaignReading`]), in store
/// order. Empty when none is active -- which is also the whole answer outside
/// every conflict's years.
///
/// `anchor_now` resolves an anchor to where it is now (the same lookup the
/// SP-4 overlay draws with), so a siege moves with the place it is attached
/// to. `gw * gh` must be the recorded rasters' length for the front and the
/// changes to be read; a snapshot of another size reads as unreadable.
pub fn campaigns_at(
    timeline: &[TimelineSnapshot],
    conflicts: &[Conflict],
    anchor_now: impl Fn(ConflictAnchor) -> Option<(f64, f64)>,
    gw: usize,
    gh: usize,
    km_per_cell: f64,
    year: i64,
) -> Vec<CampaignReading> {
    // One reconstruction per distinct recorded year, however many conflicts
    // read it.
    let mut rasters: BTreeMap<i64, Option<Vec<i32>>> = BTreeMap::new();
    let mut raster = |y: Option<i64>| -> Option<Vec<i32>> {
        let y = y?;
        rasters
            .entry(y)
            .or_insert_with(|| civ_territory_at(timeline, y).filter(|t| t.len() == gw * gh && gw > 0))
            .clone()
    };
    let territory_year = year_in_force(timeline, year);
    let now = raster(territory_year);

    conflicts
        .iter()
        .filter(|c| c.active_in(year))
        .map(|c| {
            let baseline_year = year_in_force(timeline, c.start_year);
            let before = raster(baseline_year);
            let is_side = |f: i32| c.sides.contains(&f);
            let siege = (c.kind == ConflictKind::Siege).then(|| {
                let pts = c.resolved_points(c.anchor.and_then(&anchor_now));
                let centre = pts.first().copied().unwrap_or((f64::NAN, f64::NAN));
                let defender = now.as_ref().and_then(|t| {
                    let (x, y) = (centre.0.floor(), centre.1.floor());
                    if !(x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh) {
                        return None;
                    }
                    Some(t[y as usize * gw + x as usize]).filter(|&o| is_side(o))
                });
                SiegeRing {
                    centre,
                    radius_cells: siege_line_radius_cells(km_per_cell),
                    besieged_tid: match c.anchor {
                        Some(ConflictAnchor::Settlement(t)) => Some(t),
                        _ => None,
                    },
                    defender,
                    besiegers: defender.map(|d| c.sides.iter().copied().filter(|&s| s != d).collect()),
                }
            });
            let front_edges = now.as_ref().map(|t| front_edges(t, gw, &c.sides));
            let front_cells = front_edges.as_ref().map(|e| {
                let mut v: Vec<u32> = e.iter().flat_map(|&(a, b)| [a, b]).collect();
                v.sort_unstable();
                v.dedup();
                v
            });
            let changed = match (&before, &now) {
                (Some(b), Some(n)) => Some(
                    b.iter()
                        .zip(n)
                        .enumerate()
                        .filter(|&(_, (&f, &t))| f != t && is_side(f) && is_side(t))
                        .map(|(i, (&f, &t))| ChangedCell { cell: i as u32, from: f, to: t })
                        .collect(),
                ),
                _ => None,
            };
            CampaignReading {
                conflict_id: c.id,
                sides: c.sides.clone(),
                territory_year,
                baseline_year,
                siege,
                front_edges,
                front_cells,
                changed,
            }
        })
        .collect()
}

/// 4-neighbour edges between two different sides, ascending by `(a, b)`.
fn front_edges(t: &[i32], gw: usize, sides: &[i32]) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for i in 0..t.len() {
        let a = t[i];
        if !sides.contains(&a) {
            continue;
        }
        let x = i % gw;
        // Right neighbour, then the one below: each edge is visited once,
        // from its lower index.
        for j in [(x + 1 < gw).then_some(i + 1), Some(i + gw).filter(|&j| j < t.len())].into_iter().flatten() {
            let b = t[j];
            if b != a && sides.contains(&b) {
                out.push((i as u32, j as u32));
            }
        }
    }
    out
}

/// A front edge as the line segment it is on the map, in grid coordinates
/// (cell `(x, y)` spans `[x, x+1) x [y, y+1)`): the shared side of the two
/// cells. `None` for a pair that is not a right or lower neighbour.
pub fn edge_segment(a: u32, b: u32, gw: usize) -> Option<((f64, f64), (f64, f64))> {
    let (a, b) = (a as usize, b as usize);
    let (x, y) = ((a % gw) as f64, (a / gw) as f64);
    if b == a + 1 && a % gw + 1 < gw {
        Some(((x + 1.0, y), (x + 1.0, y + 1.0)))
    } else if b == a + gw {
        Some(((x, y + 1.0), (x + 1.0, y + 1.0)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::civ_snapshot_save;

    const GW: usize = 6;
    const GH: usize = 4;

    /// Year 100: columns 0-2 are faction 1, columns 3-5 faction 2, except
    /// cell (5,0), which is faction 3. Year 110: column 3 has gone to faction
    /// 1, and cell (5,3) has gone from 2 to faction 3 -- a change that does
    /// not involve both of the conflict's sides.
    fn world() -> Vec<TimelineSnapshot> {
        let mut y100 = vec![0; GW * GH];
        for y in 0..GH {
            for x in 0..GW {
                y100[y * GW + x] = if x < 3 { 1 } else { 2 };
            }
        }
        y100[5] = 3;
        let mut y110 = y100.clone();
        for y in 0..GH {
            y110[y * GW + 3] = 1;
        }
        y110[3 * GW + 5] = 3;
        let mut tl = Vec::new();
        civ_snapshot_save(&mut tl, 100, y100, Vec::new(), Vec::new());
        civ_snapshot_save(&mut tl, 110, y110, Vec::new(), Vec::new());
        tl
    }

    /// A siege of the town `tid` 7, drawn at (4.5, 1.5) when the town stood
    /// at (4, 1); the town has since moved to (4, 2).
    fn siege() -> Conflict {
        Conflict {
            id: 5,
            name: "Siege of Varr".into(),
            kind: ConflictKind::Siege,
            start_year: 100,
            end_year: Some(120),
            sides: vec![1, 2],
            outcome: String::new(),
            points: vec![(4.5, 1.5)],
            anchor: Some(ConflictAnchor::Settlement(7)),
            anchor_at: (4.0, 1.0),
        }
    }

    fn town_now(a: ConflictAnchor) -> Option<(f64, f64)> {
        (a == ConflictAnchor::Settlement(7)).then_some((4.0, 2.0))
    }

    #[test]
    fn an_in_conflict_year_reads_the_ring_the_front_and_the_changes() {
        let got = campaigns_at(&world(), &[siege()], town_now, GW, GH, 0.5, 110);
        assert_eq!(got.len(), 1);
        let r = &got[0];
        assert_eq!(r.conflict_id, 5);
        assert_eq!(r.territory_year, Some(110));
        assert_eq!(r.baseline_year, Some(100));

        let s = r.siege.as_ref().unwrap();
        // Moved with its town: (4.5, 1.5) + (0, 1).
        assert_eq!(s.centre, (4.5, 2.5));
        // 11 x 1.48 km / 2 pi = 2.591 042 km, over 0.5 km per cell.
        assert!((s.radius_cells.unwrap() - 5.182_084).abs() < 1e-6, "{:?}", s.radius_cells);
        assert_eq!(s.besieged_tid, Some(7));
        // Cell (4, 2) is faction 2's in 110.
        assert_eq!(s.defender, Some(2));
        assert_eq!(s.besiegers, Some(vec![1]));

        // Column 3 (faction 1) against column 4 (faction 2), one edge per row;
        // (4,0)|(5,0) is 2|3 and (4,3)|(5,3) is 2|3 -- not this war's.
        assert_eq!(r.front_edges.as_ref().unwrap(), &vec![(3, 4), (9, 10), (15, 16), (21, 22)]);
        assert_eq!(r.front_cells.as_ref().unwrap().len(), 8);
        assert_eq!(r.front_cells.as_ref().unwrap(), &vec![3, 4, 9, 10, 15, 16, 21, 22]);

        // Column 3 went 2 -> 1; (5,3) went 2 -> 3 and is not counted.
        let changed = r.changed.as_ref().unwrap();
        assert_eq!(
            changed,
            &[3u32, 9, 15, 21].map(|cell| ChangedCell { cell, from: 2, to: 1 }).to_vec()
        );
    }

    #[test]
    fn an_unrecorded_year_reads_the_year_in_force() {
        // 105: nothing recorded, so 100's claims hold -- the front is between
        // columns 2 and 3, and nothing has changed hands yet.
        let got = campaigns_at(&world(), &[siege()], town_now, GW, GH, 0.5, 105);
        let r = &got[0];
        assert_eq!(r.territory_year, Some(100));
        assert_eq!(r.front_edges.as_ref().unwrap(), &vec![(2, 3), (8, 9), (14, 15), (20, 21)]);
        assert_eq!(r.changed.as_ref().unwrap(), &Vec::<ChangedCell>::new());
    }

    #[test]
    fn outside_the_conflicts_years_there_is_nothing() {
        for y in [99, 121, 10_000, -5] {
            assert!(campaigns_at(&world(), &[siege()], town_now, GW, GH, 0.5, y).is_empty(), "year {y}");
        }
        // And on the boundary years, there is.
        assert_eq!(campaigns_at(&world(), &[siege()], town_now, GW, GH, 0.5, 100).len(), 1);
        assert_eq!(campaigns_at(&world(), &[siege()], town_now, GW, GH, 0.5, 120).len(), 1);
    }

    #[test]
    fn a_reading_that_cannot_be_made_is_absent_not_empty() {
        // Starts before anything is recorded: no baseline, so no changes --
        // but the front at 110 is still readable.
        let early = Conflict { start_year: 90, ..siege() };
        let r = &campaigns_at(&world(), &[early.clone()], town_now, GW, GH, 0.5, 110)[0];
        assert_eq!(r.baseline_year, None);
        assert_eq!(r.changed, None);
        assert!(r.front_edges.is_some());
        // At 95 nothing is recorded at all.
        let r = &campaigns_at(&world(), &[early], town_now, GW, GH, 0.5, 95)[0];
        assert_eq!((r.territory_year, r.front_edges.clone(), r.front_cells.clone()), (None, None, None));
        assert_eq!(r.siege.as_ref().unwrap().defender, None);
        assert_eq!(r.siege.as_ref().unwrap().besiegers, None);
        // A grid of another size cannot be read as this one.
        let r = &campaigns_at(&world(), &[siege()], town_now, GW + 1, GH, 0.5, 110)[0];
        assert_eq!((r.front_edges.clone(), r.changed.clone()), (None, None));
        // No map scale: the ring's size is unknown, not a default.
        assert_eq!(siege_line_radius_cells(0.0), None);
        assert_eq!(siege_line_radius_cells(f64::NAN), None);
    }

    #[test]
    fn a_coarse_map_floors_the_ring_at_one_cell() {
        // 2.591 km over 10 km per cell is 0.259 cells -- inside the cell.
        assert_eq!(siege_line_radius_cells(10.0), Some(1.0));
        // At 1 km per cell the attested radius stands.
        assert!((siege_line_radius_cells(1.0).unwrap() - 2.591_042).abs() < 1e-6);
    }

    #[test]
    fn only_sieges_carry_a_ring_and_an_unanchored_one_names_no_place() {
        let front = Conflict { kind: ConflictKind::Front, points: vec![(0.0, 0.0), (1.0, 1.0)], ..siege() };
        let r = &campaigns_at(&world(), &[front], town_now, GW, GH, 0.5, 110)[0];
        assert_eq!(r.siege, None);
        assert_eq!(r.front_edges.as_ref().unwrap().len(), 4);
        // Free-standing: drawn where drawn, no besieged place named. (0.5,
        // 0.5) is faction 1's cell in 110.
        let free = Conflict { anchor: None, points: vec![(0.5, 0.5)], ..siege() };
        let s = campaigns_at(&world(), &[free], town_now, GW, GH, 0.5, 110)[0].siege.clone().unwrap();
        assert_eq!((s.centre, s.besieged_tid, s.defender), ((0.5, 0.5), None, Some(1)));
        assert_eq!(s.besiegers, Some(vec![2]));
        // Under a third party's cell (5,0): the defender is not a side.
        let third = Conflict { anchor: None, points: vec![(5.5, 0.5)], ..siege() };
        let s = campaigns_at(&world(), &[third], town_now, GW, GH, 0.5, 110)[0].siege.clone().unwrap();
        assert_eq!((s.defender, s.besiegers), (None, None));
    }

    #[test]
    fn a_one_sided_conflict_has_no_front_and_no_changes() {
        let lone = Conflict { sides: vec![1], ..siege() };
        let r = &campaigns_at(&world(), &[lone], town_now, GW, GH, 0.5, 110)[0];
        assert_eq!(r.front_edges.as_ref().unwrap(), &Vec::<(u32, u32)>::new());
        assert_eq!(r.changed.as_ref().unwrap(), &Vec::<ChangedCell>::new());
    }

    #[test]
    fn an_edge_is_the_shared_side_of_its_two_cells() {
        // (3,0)|(4,0): the vertical line x = 4 from y = 0 to 1.
        assert_eq!(edge_segment(3, 4, GW), Some(((4.0, 0.0), (4.0, 1.0))));
        // (3,1) over (3,2): the horizontal line y = 2 from x = 3 to 4.
        assert_eq!(edge_segment(9, 15, GW), Some(((3.0, 2.0), (4.0, 2.0))));
        // (5,0) and (0,1) are index-adjacent but not neighbours.
        assert_eq!(edge_segment(5, 6, GW), None);
        assert_eq!(edge_segment(3, 5, GW), None);
    }
}

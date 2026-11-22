//! Finding F26 under owner Ruling GL (2026-11-22, `LARGE_ITEM_RULINGS.md`):
//! **city-states and their spheres of influence.**
//!
//! A sizeable unclaimed settlement ([`is_city_state`]) is a city-state. Each
//! city-state projects a sphere of influence over unclaimed land, computed by
//! the same cost-distance competition as faction territory
//! (`crate::assign_territory`): effective distance is raw cost-distance over
//! the territory cost (the plain road cost, or Ruling GO's friction with its
//! river edge cost when the caller passes one) divided by
//! `crate::territory_weight(pop)`, so a larger city reaches farther for the
//! same terrain. Loose settlements standing inside a sphere are **aligned** to
//! it; they are never made a faction here (Ruling GL: "the user decides when a
//! cluster becomes a faction"). The shell's promotion turns a sphere into a
//! faction through the roster, which is the only route to one.
//!
//! **Why one multi-source sweep, not one Dijkstra per seat.** Territory runs
//! one full-grid Dijkstra per capital and takes the per-cell minimum, so a
//! large capital can leapfrog a small one. A sphere is a bounded thing (it
//! stops at [`SPHERE_REACH_FRAC`]) and must not pass through another sphere,
//! claimed land or water, so the search runs once from every seat at once,
//! each seat's steps scaled by its own `1 / weight`, and a cell belongs to
//! the first seat whose front reaches it (a growth model: every sphere is
//! connected through land it holds). It visits only the cells it labels,
//! which is what lets the shell recompute spheres for every recorded year on
//! demand rather than hold a grid per year.
//!
//! **Not the reference.** The reference has no city-states, no spheres and no
//! algorithmic territory at all, so there is no golden to match; this is new
//! design under `DECISIONS.md` §7p, gated by `CivParams::city_states` at the
//! app boundary (read live, never inside `compute_civilisation`, so no
//! generated output moves).
//!
//! What it must never do: label a cell a faction owns, align a settlement
//! that belongs to a faction, align a city-state to another city-state, or
//! cross water.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::NamedSettlement;

/// The population at or above which an unclaimed settlement is a
/// city-state. **A labelled judgement, placed by measurement**: on the
/// app-default worlds of seeds 483920, 912345 and 246371 at grid widths 384
/// and 2048 (`cartalith-godot`'s `city_states_measure`, output in
/// `cartalith-native/docs/evidence/city-states-2026-11-22/measure.txt`) the
/// placed populations fall in clear tiers: every town measured was at most
/// 2 609 people and every city at least 7 316 (capitals 16 155 to 24 962).
/// 5 000 sits inside that gap, so the cities and capitals of a world with no
/// faction are city-states and its towns, villages and hamlets are not.
pub const CITY_STATE_MIN_POP: u32 = 5_000;

/// How far a sphere reaches, as a fraction of the grid width, in effective
/// cost-distance (raw cost-distance over the territory cost divided by the
/// seat's `territory_weight`). The cost is about 1 per cell on flat open land
/// (`build_travel_cost` is `1 + 50 slope^2`), so on flat land a seat of
/// weight `w` reaches about `w * gw * SPHERE_REACH_FRAC` cells, and a grid-
/// relative reach keeps one world's spheres alike at 384 and at 2048.
/// **A labelled judgement, picked by measurement** (the same harness and
/// evidence file; this sweep was run before F27's junction hamlets landed,
/// which only add hamlets to align): in the three zero-faction worlds at 384, a twelfth aligned
/// 32 of 62 other loose places with 29-37 % of the land in a sphere, an eighth
/// 46 of 62 with 54-59 %, and a tenth 39 of 62 with 39-47 % (at 2048: 61 of
/// 99, 42-57 %). A tenth is chosen: most places near a city-state are drawn
/// in, and unclaimed frontier still separates the spheres, so a cluster reads
/// as a sphere rather than as a partition of the map.
pub const SPHERE_REACH_FRAC: f64 = 1.0 / 10.0;

/// Whether settlement `s` is a city-state: unclaimed (faction `0`) and at
/// least [`CITY_STATE_MIN_POP`] people.
pub fn is_city_state(s: &NamedSettlement) -> bool {
    s.placement.faction == 0 && s.pop >= CITY_STATE_MIN_POP
}

/// The spheres of one world state, all indexed like the inputs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spheres {
    /// The settlement index of every city-state whose own cell is unclaimed
    /// land, in settlement order. A sphere's id is its position here plus 1.
    pub seats: Vec<usize>,
    /// Per cell: `0` outside every sphere, else the sphere id (`seats`
    /// index plus 1). Empty when the inputs could not be read (see
    /// [`spheres`]).
    pub cell: Vec<u32>,
    /// Per settlement: the settlement index of the seat it is aligned to, or
    /// `None`. A seat is `Some(itself)`; a faction's settlement is always
    /// `None`.
    pub aligned: Vec<Option<usize>>,
}

impl Spheres {
    /// How many cells sphere `k` (a `seats` index) holds.
    pub fn cells_of(&self, k: usize) -> usize {
        let id = k as u32 + 1;
        self.cell.iter().filter(|&&c| c == id).count()
    }

    /// The settlement indices aligned to seat `seat` (a settlement index),
    /// the seat itself excluded.
    pub fn members_of(&self, seat: usize) -> Vec<usize> {
        self.aligned
            .iter()
            .enumerate()
            .filter(|&(i, a)| *a == Some(seat) && i != seat)
            .map(|(i, _)| i)
            .collect()
    }
}

/// A heap entry: effective distance, sphere index, cell. Ordered as a min-heap
/// on distance, then the lower sphere index, then the lower cell, so ties are
/// settled the same way on every run.
#[derive(PartialEq)]
struct Front(f64, usize, usize);
impl Eq for Front {}
impl Ord for Front {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0).then(o.1.cmp(&self.1)).then(o.2.cmp(&self.2))
    }
}
impl PartialOrd for Front {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// The spheres of influence of `settlements` over `owner` (the faction claim
/// grid, `0` = unclaimed). `cost` is the per-cell territory cost (infinite on
/// water); `edge`, when given, is the per-step cost the territory sweep used
/// (Ruling GO's `Friction::edge`), else the default `len * (c_i + c_j) / 2`.
///
/// Returns an empty [`Spheres`] (no seats, no cells, every settlement `None`)
/// when `owner` or `cost` is not a whole `gw * gh` grid: without a claim grid
/// there is no knowing which land is unclaimed, and treating it as all
/// unclaimed would let a sphere take a faction's land.
pub fn spheres(
    settlements: &[NamedSettlement],
    owner: &[i32],
    cost: &[f32],
    edge: Option<&(dyn Fn(usize, usize, isize, isize) -> f64 + Sync)>,
    gw: usize,
    gh: usize,
    world: bool,
) -> Spheres {
    let n = gw * gh;
    let none = Spheres { aligned: vec![None; settlements.len()], ..Default::default() };
    if n == 0 || owner.len() != n || cost.len() != n {
        return none;
    }
    let cell_of = |s: &NamedSettlement| (s.placement.x < gw && s.placement.y < gh).then(|| s.placement.y * gw + s.placement.x);
    let open = |i: usize| owner[i] == 0 && cost[i].is_finite();
    let seats: Vec<usize> = settlements
        .iter()
        .enumerate()
        .filter(|(_, s)| is_city_state(s) && cell_of(s).is_some_and(open))
        .map(|(i, _)| i)
        .collect();
    let reach = gw as f64 * SPHERE_REACH_FRAC;
    let mut label = vec![0u32; n];
    let mut best = vec![f64::INFINITY; n];
    let mut heap = BinaryHeap::new();
    let weights: Vec<f64> = seats.iter().map(|&i| crate::territory_weight(settlements[i].pop)).collect();
    for (k, &si) in seats.iter().enumerate() {
        let c = cell_of(&settlements[si]).expect("seats were filtered on a cell");
        // Two seats on one cell: the first in settlement order holds it.
        if best[c] > 0.0 {
            best[c] = 0.0;
            heap.push(Front(0.0, k, c));
        }
    }
    while let Some(Front(d, k, i)) = heap.pop() {
        if label[i] != 0 || d > best[i] {
            continue;
        }
        label[i] = k as u32 + 1;
        let (x, y) = ((i % gw) as isize, (i / gw) as isize);
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = if world {
                    (x + dx).rem_euclid(gw as isize)
                } else if x + dx < 0 || x + dx >= gw as isize {
                    continue;
                } else {
                    x + dx
                };
                let ny = y + dy;
                if ny < 0 || ny >= gh as isize {
                    continue;
                }
                let j = ny as usize * gw + nx as usize;
                if label[j] != 0 || !open(j) {
                    continue;
                }
                let step = match edge {
                    Some(f) => f(i, j, dx, dy),
                    None => (if dx != 0 && dy != 0 { std::f64::consts::SQRT_2 } else { 1.0 }) * 0.5 * (cost[i] as f64 + cost[j] as f64),
                };
                let nd = d + step / weights[k];
                if nd <= reach && nd < best[j] {
                    best[j] = nd;
                    heap.push(Front(nd, k, j));
                }
            }
        }
    }
    let aligned = settlements
        .iter()
        .map(|s| {
            if s.placement.faction != 0 {
                return None;
            }
            let id = label[cell_of(s)?];
            (id != 0).then(|| seats[id as usize - 1])
        })
        .collect();
    Spheres { seats, cell: label, aligned }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SettlementKind, SettlementPlacement};

    fn place(x: usize, y: usize, pop: u32, faction: i32) -> NamedSettlement {
        NamedSettlement {
            tid: 0,
            placement: SettlementPlacement { x, y, suit: 0.0, faction, capital: false, kind: SettlementKind::Town, coastal: false },
            name: String::new(),
            pop,
        }
    }

    /// Protects the threshold as a literal: 5 000 people is a city-state,
    /// 4 999 is not, and a faction's settlement of any size never is.
    /// MUTATION: `>=` to `>` or 5_000 to 6_000 fails the first assertion.
    #[test]
    fn city_state_is_a_sizeable_unclaimed_settlement() {
        assert!(is_city_state(&place(0, 0, 5_000, 0)));
        assert!(!is_city_state(&place(0, 0, 4_999, 0)));
        assert!(!is_city_state(&place(0, 0, 50_000, 2)));
    }

    /// Protects the reach as a literal: on a flat 240-wide grid (cost 1) a
    /// seat of weight `w` reaches `240 / 10 * w` cells along a row and no
    /// farther. MUTATION: 1/10 to 1/12 (or 1/8) moves the edge.
    #[test]
    fn reach_is_a_tenth_of_the_width_times_the_weight() {
        let (gw, gh) = (240, 1);
        let cost = vec![1.0f32; gw];
        let owner = vec![0; gw];
        let s = vec![place(0, 0, 5_000, 0)];
        let sp = spheres(&s, &owner, &cost, None, gw, gh, false);
        let w = crate::territory_weight(5_000);
        let want = (24.0 * w).floor() as usize + 1;
        assert_eq!(want, 31, "premise: w(5 000) = 1.288, so 30.9 cells");
        assert_eq!(sp.cells_of(0), want, "weight {w}");
    }

    /// Protects Ruling GL's "never takes faction land": a sphere stops at a
    /// claimed cell even with open land past it, never labels a claimed cell,
    /// and never aligns a faction's settlement standing on unclaimed land
    /// inside it. MUTATION: drop `owner[i] == 0` from `open` and the wall is
    /// crossed; drop the faction check in `aligned` and the faction town
    /// aligns.
    #[test]
    fn a_sphere_never_takes_faction_land_or_settlements() {
        let (gw, gh) = (240, 1);
        let cost = vec![1.0f32; gw];
        let mut owner = vec![0; gw];
        owner[5] = 3;
        let s = vec![place(0, 0, 5_000, 0), place(2, 0, 100, 1), place(8, 0, 100, 0), place(3, 0, 100, 0)];
        let sp = spheres(&s, &owner, &cost, None, gw, gh, false);
        assert_eq!(sp.cell[5], 0);
        assert!(sp.cell[6..].iter().all(|&c| c == 0), "nothing past the wall");
        assert_eq!(sp.aligned, vec![Some(0), None, None, Some(0)]);
        assert_eq!(sp.members_of(0), vec![3]);
    }

    /// Protects the competition: two seats on one row split it, the larger
    /// one (higher weight) reaching farther toward the smaller; a seat on
    /// claimed land has no sphere; a missing claim grid gives no spheres.
    /// MUTATION: drop `/ weights[k]` and the split moves to the midpoint.
    #[test]
    fn spheres_compete_by_weight_and_need_a_claim_grid() {
        let (gw, gh) = (480, 1);
        let cost = vec![1.0f32; gw];
        let owner = vec![0; gw];
        let s = vec![place(0, 0, 60_000, 0), place(20, 0, 5_000, 0)];
        let sp = spheres(&s, &owner, &cost, None, gw, gh, false);
        let first_of_b = sp.cell.iter().position(|&c| c == 2).unwrap();
        assert!(first_of_b > 10, "the larger seat reaches past the midpoint: {first_of_b}");
        let mut claimed = owner.clone();
        claimed[20] = 1;
        assert_eq!(spheres(&s, &claimed, &cost, None, gw, gh, false).seats, vec![0]);
        let none = spheres(&s, &[], &cost, None, gw, gh, false);
        assert!(none.seats.is_empty() && none.cell.is_empty() && none.aligned == vec![None, None]);
    }

    /// Protects water as a wall: an infinite-cost cell is never labelled and
    /// never crossed.
    #[test]
    fn a_sphere_never_crosses_water() {
        let (gw, gh) = (240, 1);
        let mut cost = vec![1.0f32; gw];
        cost[3] = f32::INFINITY;
        let s = vec![place(0, 0, 5_000, 0), place(5, 0, 100, 0)];
        let sp = spheres(&s, &vec![0; gw], &cost, None, gw, gh, false);
        assert_eq!(sp.cell[3], 0);
        assert_eq!(sp.aligned[1], None);
    }
}

//! **Ruling HS (owner, 2026-11-23): where a coastal or floodplain town's market stands.**
//! `COASTAL_SETBACK_RESEARCH.md` sections 3.2 to 3.4, built from CS-1 on.
//!
//! - **Coast** (`urban_coastal_setback`): a town whose box holds sea has its market at the water
//!   ([`COAST_FRONT_M`] from the sea), unless that point has a hazard reason (storm surge on an exposed shore, a
//!   cliff edge, marsh or flood ground). Then the market stands on the nearest reason-free ground no more than
//!   [`COAST_CAP_M`] from the sea; with none, it stays at the water and the town records "no safe ground in
//!   reach". The quay stays at the water (`crate::water::build_harbour` reads the shore, not the market) and a
//!   harbour street joins the two (`crate::generate::harbour_street`).
//! - **River** (`urban_floodplain_setback`): the bank default of Rulings HG, HM and HN stands. A river town's
//!   market moves only where it stands on floodplain (its grid cell flagged, and under [`H_FLOOD_M`] above its
//!   channel by the in-box heights): straight away from the river, on its own bank, to the first ground at least
//!   [`H_FLOOD_M`] above the channel and not wetland, within [`RIVER_CAP_M`] (section 3.4's terrace walk). With
//!   none in reach the market stays and the town records "no terrace in reach".
//! - **Coarse grids** ([`SetbackCtx::fine`] false): the per-cell fields are one value over the whole box, so they
//!   cannot say where to move; nothing moves, and a floodplain river town records "no terrace in reach".
//!
//! The host (`cartalith_civ::setback`) reads the world's fields and hands the hazard reasons over as a raster on
//! the town's own 22 m water raster ([`SetbackCtx`]). This module only chooses points. **No reason means no
//! record**: a town whose market came to the water has no [`Setback`] at all, never one of 0 m (`MISTAKES.md`).
//!
//! Must never move a market onto water, across its river, or farther than its cap; and must never act without a
//! [`SetbackCtx`] (`None` is the layout before the ruling, by control flow).

use crate::geom::{Vec2, dist_pt_seg};

/// Hazard bit: storm surge (low ground near an exposed sea).
pub const SURGE: u8 = 1;
/// Hazard bit: within reach of a cliff top over the sea.
pub const CLIFF: u8 = 2;
/// Hazard bit: marsh or flood ground (the cell's wetland mask, or its flood field over the reference's line).
pub const MARSH: u8 = 4;
/// Cell flag: the grid cell is floodplain (landform class 6, or flood field over the line). Not a reason alone:
/// the river rule also asks the in-box heights.
pub const FLOODPLAIN: u8 = 8;
/// Cell flag: the grid cell is wetland. A terrace the river walk reaches must not be.
pub const WETLAND: u8 = 16;
/// Raster flag: the cell is sea (lakes excluded, research Q5).
pub const SEA: u8 = 32;
/// The three coast reasons together.
pub const COAST_REASONS: u8 = SURGE | CLIFF | MARSH;

/// How far from the sea raster a coastal market with no reason stands, metres. **A labelled judgement**: the
/// engine's market-to-harbour distance (`crate::generate::HARBOUR_FRONT_M`, 100 m) less one raster cell's diagonal
/// (22 m x 1.414, 31 m, rounded down to 30). The host's raster calls a cell sea where the map draws sea at its
/// centre or any corner (Ruling HN), so the raster's sea edge stands up to a diagonal landward of the drawn shore:
/// CS-0 measured the markets `harbour_front_market` set 100 m from the raster at 103 to 128 m from the drawn sea
/// (`COASTAL_SETBACK_RESEARCH.md` section 7.2, class (d)). At 70 m the market is within 100 m of the drawn sea.
pub const COAST_FRONT_M: f64 = 70.0;
/// The coastal setback's cap, metres from the sea: `D_CAP_BOX` (research 3.2, Q2; owner Ruling HS "capped at
/// 600 m"). A judgement: about half the box's short side, so the quay and the core both stay in the box.
pub const COAST_CAP_M: f64 = 600.0;
/// The river setback's cap, metres the market may walk from where it stood: `D_CAP_RIVER` (research 3.4, Q2;
/// owner Ruling HS "up to 500 m"). Inside the box's 625 m half-height.
pub const RIVER_CAP_M: f64 = 500.0;
/// `H_FLOOD`, metres above the channel that count as above flood level (research 3.4: "at least 2 m", the floor of
/// the bankfull depth; CS-0 swept 2, 4 and 8 m and the walk found a terrace for 40, 39 and 35 of 51 towns at
/// 2 048). A labelled judgement at the research's floor.
pub const H_FLOOD_M: f64 = 2.0;
/// The river walk's step, metres: half the 22 m raster, so no raster cell is stepped over.
pub const WALK_STEP_M: f64 = 11.0;

/// The hazard reasons over one town's box, on the town's own water raster (`crate::site::WaterCtx`'s `mw` by
/// `mh` cells of `cell_m`), from the host. Built by `cartalith_civ::setback::setback_ctx`.
#[derive(Debug, Clone, Default)]
pub struct SetbackCtx {
    /// `urban_coastal_setback`: the coastal rule may act (the host sets it only for a box that holds sea).
    pub coast: bool,
    /// `urban_floodplain_setback`: the river rule may act.
    pub river: bool,
    /// Whether the world's grid resolves a hazard inside the box (a 2 048-class grid). `false`: nothing moves.
    pub fine: bool,
    pub mw: usize,
    pub mh: usize,
    pub cell_m: f64,
    /// Per raster cell: [`SURGE`], [`CLIFF`], [`MARSH`], [`FLOODPLAIN`], [`WETLAND`], [`SEA`].
    pub bits: Vec<u8>,
    /// Per raster cell: ground height above sea, metres (the bilinear of the world grid at the cell centre).
    pub ground_m: Vec<f32>,
    /// Per raster cell: distance from the cell centre to the nearest sea cell's edge, metres (0 on sea;
    /// `INFINITY` with no sea in the box, never a plausible distance).
    pub sea_m: Vec<f32>,
}

/// Which water a [`Setback`] is from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetbackWater {
    Coast,
    River,
}

/// **A town the rule set back, or would have and could not.** Present only where a reason was found at the
/// market (no reason: no record).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setback {
    pub water: SetbackWater,
    /// The reasons at the point the market would otherwise have stood ([`COAST_REASONS`] bits, or
    /// [`FLOODPLAIN`] on a river).
    pub reasons: u8,
    /// Where the market went and how far that is from its water, metres; `None` where no reason-free ground was
    /// in reach (coast: "no safe ground in reach"; river or a coarse grid: "no terrace in reach"), and the market
    /// stayed.
    pub to: Option<(Vec2, f64)>,
}

impl Setback {
    /// The record as a short phrase for Inspect and the census: the reasons and the outcome.
    pub fn describe(&self) -> String {
        let mut r: Vec<&str> = Vec::new();
        for (b, w) in [(SURGE, "storm surge"), (CLIFF, "cliff edge"), (MARSH, "marsh or flood"), (FLOODPLAIN, "floodplain")] {
            if self.reasons & b != 0 {
                r.push(w);
            }
        }
        match (self.to, self.water) {
            (Some((_, m)), _) => format!("set back {m:.0} m ({})", r.join(", ")),
            (None, SetbackWater::Coast) => format!("no safe ground in reach ({})", r.join(", ")),
            (None, SetbackWater::River) => format!("no terrace in reach ({})", r.join(", ")),
        }
    }
}

impl SetbackCtx {
    /// The raster cell holding `p` (box metres), or `None` outside the raster or for a raster of the wrong size.
    pub fn cell(&self, p: Vec2) -> Option<usize> {
        if self.bits.len() != self.mw * self.mh || !(self.cell_m > 0.0) || p.x < 0.0 || p.y < 0.0 {
            return None;
        }
        let (i, j) = ((p.x / self.cell_m) as usize, (p.y / self.cell_m) as usize);
        (i < self.mw && j < self.mh).then_some(j * self.mw + i)
    }

    fn centre(&self, k: usize) -> Vec2 {
        Vec2::new(((k % self.mw) as f64 + 0.5) * self.cell_m, ((k / self.mw) as f64 + 0.5) * self.cell_m)
    }
}

/// **The coastal rule**: the market for a town whose box centre is `mc`, and its record. `None` where the rule
/// does not act (the switch off, a coarse grid, no sea in the raster): the caller then places the market as before.
///
/// `q0` is the point [`COAST_FRONT_M`] from the nearest sea cell (its square, ties to the first in row order) on
/// the line from `mc`, or `mc` itself where it is nearer. With no reason at `q0` the market is `q0` and there is
/// no record. Otherwise it is the reason-free land cell centre (`wet`, the site's water test, false; no [`COAST_REASONS`]
/// bit) at most [`COAST_CAP_M`] from the sea that is nearest `q0` (ties: the higher ground, the defensible-site
/// preference of research 3.2, then row order), recorded with its distance from the sea; with none, `q0` with
/// "no safe ground in reach".
///
/// Returns `(market, sea point the market faces, record)`; the sea point is for the harbour street.
pub fn coast_market(sc: &SetbackCtx, mc: Vec2, wet: &dyn Fn(Vec2) -> bool) -> Option<(Vec2, Vec2, Option<Setback>)> {
    if !sc.coast || !sc.fine || sc.bits.len() != sc.mw * sc.mh {
        return None;
    }
    let c = sc.cell_m;
    let nearest_sea = |p: Vec2| -> Option<Vec2> {
        let mut best: Option<(f64, Vec2)> = None;
        for k in 0..sc.bits.len() {
            if sc.bits[k] & SEA == 0 {
                continue;
            }
            let (x0, y0) = ((k % sc.mw) as f64 * c, (k / sc.mw) as f64 * c);
            let q = Vec2::new(p.x.clamp(x0, x0 + c), p.y.clamp(y0, y0 + c));
            let d = q.dist(p);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, q));
            }
        }
        best.map(|(_, q)| q)
    };
    let q = nearest_sea(mc)?;
    let q0 = if mc.dist(q) > COAST_FRONT_M { q + (mc - q).norm() * COAST_FRONT_M } else { mc };
    let reasons = sc.cell(q0).map_or(0, |k| sc.bits[k] & COAST_REASONS);
    if reasons == 0 {
        return Some((q0, q, None));
    }
    let mut best: Option<(f64, f32, usize)> = None;
    for k in 0..sc.bits.len() {
        let b = sc.bits[k];
        if b & (SEA | COAST_REASONS) != 0 || !(f64::from(sc.sea_m[k]) <= COAST_CAP_M) {
            continue;
        }
        let p = sc.centre(k);
        if wet(p) {
            continue;
        }
        let d = p.dist(q0);
        let h = sc.ground_m[k];
        let better = match best {
            None => true,
            Some((bd, bh, _)) => d < bd || (d == bd && h > bh),
        };
        if better {
            best = Some((d, h, k));
        }
    }
    Some(match best {
        Some((_, _, k)) => {
            let p = sc.centre(k);
            (p, nearest_sea(p).unwrap_or(q), Some(Setback { water: SetbackWater::Coast, reasons, to: Some((p, f64::from(sc.sea_m[k]))) }))
        }
        None => (q0, q, Some(Setback { water: SetbackWater::Coast, reasons, to: None })),
    })
}

/// **The river rule** (research 3.4's terrace walk): for a river town whose market stands at `mc`, the market
/// and its record, or `None` where the rule does not act (the switch off, no river centreline, a market not on
/// floodplain: its cell unflagged or its ground at least [`H_FLOOD_M`] above the channel).
///
/// `river` is the site's centreline (box metres) and `wet` its water test (`Site::is_water`).
/// The channel's height is the ground at the centreline point nearest `mc`. The walk goes from `mc` straight
/// away from that point (so on the market's own bank) in [`WALK_STEP_M`] steps up to [`RIVER_CAP_M`], and stops at
/// the first point at least [`H_FLOOD_M`] above the channel whose cell is not [`WETLAND`]; water or the box's edge
/// first ends it with "no terrace in reach", as does a coarse grid ([`SetbackCtx::fine`] false) without a walk.
pub fn river_market(sc: &SetbackCtx, river: &[Vec2], mc: Vec2, wet: &dyn Fn(Vec2) -> bool) -> Option<(Vec2, Setback)> {
    if !sc.river || river.len() < 2 {
        return None;
    }
    let k = sc.cell(mc)?;
    if sc.bits[k] & FLOODPLAIN == 0 {
        return None;
    }
    // The foot of the perpendicular from `mc` on the centreline.
    let mut foot: Option<(f64, Vec2)> = None;
    for w in river.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b - a;
        let l2 = ab.dot(ab);
        let t = if l2 > 0.0 { ((mc - a).dot(ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let f = a + ab * t;
        let d = dist_pt_seg(mc, a, b);
        if foot.is_none_or(|(bd, _)| d < bd) {
            foot = Some((d, f));
        }
    }
    let (_, f) = foot?;
    let ground = |p: Vec2| sc.cell(p).map(|k| f64::from(sc.ground_m[k]));
    let chan = ground(f).unwrap_or(f64::from(sc.ground_m[k]));
    if f64::from(sc.ground_m[k]) - chan >= H_FLOOD_M {
        return None;
    }
    let stay = Setback { water: SetbackWater::River, reasons: FLOODPLAIN, to: None };
    if !sc.fine || mc.dist(f) < 1e-9 {
        return Some((mc, stay));
    }
    let dir = (mc - f).norm();
    let n = (RIVER_CAP_M / WALK_STEP_M).floor() as usize;
    for t in 1..=n {
        let p = mc + dir * (t as f64 * WALK_STEP_M);
        let Some(kp) = sc.cell(p) else { break };
        if wet(p) {
            break;
        }
        if f64::from(sc.ground_m[kp]) - chan >= H_FLOOD_M && sc.bits[kp] & WETLAND == 0 {
            return Some((p, Setback { water: SetbackWater::River, reasons: FLOODPLAIN, to: Some((p, p.dist(f))) }));
        }
    }
    Some((mc, stay))
}

#[cfg(test)]
mod tests;

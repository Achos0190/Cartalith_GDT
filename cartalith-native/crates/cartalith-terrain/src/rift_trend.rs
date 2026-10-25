//! The trend of a rift at a vent, read off the plate boundary at generation
//! (owner Ruling DB item 1, 2026-10-25: *"derive the rift direction at
//! generation from the plate boundary and record it in the volcano table"*).
//!
//! # What it does
//!
//! [`rift_trend`] answers, for a vent at a grid position, the bearing of the
//! divergent plate boundary it sits on: the axis along which the boundary
//! runs there, in radians in `0..PI` (an axis, so `b` and `b + PI` are the
//! same trend), measured like `atan2(dy, dx)` on the grid (x east, y south).
//! The engine records it as [`crate::PlacedEdifice::rift_bearing`] for every
//! edifice placed in a [`crate::VolcanicSetting::Rift`] province, and the lava
//! flows (`lava_flows`) lay a rift edifice's eruptive fissure and a shield's
//! rift-zone vents along it.
//!
//! # Why this shape
//!
//! Eruptive fissures and the dyke swarms that feed them open perpendicular to
//! the least compressive stress, which at a spreading boundary is the
//! spreading direction, so they run **parallel to the rift axis**: Iceland's
//! fissure swarms (Krafla, Laki, Holuhraun) trend along the neovolcanic
//! zones, and the East African rift's fissures along the rift (Gudmundsson
//! 2000, *Dynamics of volcanic systems in Iceland*, Annu. Rev. Earth Planet.
//! Sci. 28). The plate boundary is the engine's own record of that axis, so
//! the trend is read from it, not drawn at random:
//!
//! 1. **The boundary's own direction.** The cells within [`RADIUS_CELLS`] of
//!    the vent where the plate id changes to a 4-neighbour are collected and
//!    their principal axis (the eigenvector of their position covariance
//!    with the larger eigenvalue) is the trend, when at least
//!    [`MIN_CELLS`] cells are found and the spread along the axis is at least
//!    [`MIN_ANISOTROPY`] times the spread across it (a boundary, not a blob
//!    where three plates meet).
//! 2. **Otherwise the spreading direction.** Perpendicular to the relative
//!    velocity of the vent's plate and the plate across the boundary (the
//!    commonest other plate among those cells' neighbours): the same
//!    physics, read from the plates' motions instead of the boundary's shape.
//! 3. **Otherwise none.** No boundary cell within the radius: no trend is
//!    recorded, never a guessed one.
//!
//! Every input is the generation's own plate field, so the trend is a pure
//! function of the world; it reads no random stream and changes no generated
//! value (the table records it beside the edifice).
//!
//! # What it must never do
//!
//! - Never write the plate field or anything else: it reads and returns.
//! - Never return a bearing outside `0..PI` or a non-finite one.

use crate::Plate;

/// How far from the vent boundary cells are collected, in cells. Judgement:
/// at the default 800 km over 2048 cells (0.39 km a cell) about 9 km, the
/// scale of one Icelandic volcanic system's fissure swarm width (5-20 km,
/// Gudmundsson 2000), so the trend is the boundary's local run, not the
/// whole plate's.
pub const RADIUS_CELLS: i64 = 24;
/// The fewest boundary cells that make a direction. Judgement.
pub const MIN_CELLS: usize = 6;
/// The least ratio of the along-axis to the across-axis variance for the
/// boundary's own direction to be trusted. Judgement: 2 (an aspect ratio of
/// about 1.4 in standard deviations); below it the cells are a junction or a
/// tight bend and the plates' motion decides.
pub const MIN_ANISOTROPY: f64 = 2.0;

/// The rift trend at grid position `(x, y)` (module doc), or `None` when no
/// boundary lies within [`RADIUS_CELLS`]. `world` wraps the x axis (a
/// whole-world map), as the plate field does. `plate_id` is `gw * gh`; a
/// plate id with no entry in `plates` is skipped for the velocity fallback.
pub fn rift_trend(plate_id: &[u16], plates: &[Plate], gw: usize, gh: usize, world: bool, x: f64, y: f64) -> Option<f64> {
    if gw < 2 || gh < 2 || plate_id.len() < gw * gh || !(x.is_finite() && y.is_finite()) {
        return None;
    }
    let (cx, cy) = (x.round() as i64, y.round() as i64);
    let at = |xx: i64, yy: i64| -> Option<u16> {
        if yy < 0 || yy >= gh as i64 {
            return None;
        }
        let xx = if world { xx.rem_euclid(gw as i64) } else if xx < 0 || xx >= gw as i64 { return None } else { xx };
        Some(plate_id[yy as usize * gw + xx as usize])
    };
    let own = at(cx.clamp(0, gw as i64 - 1), cy.clamp(0, gh as i64 - 1))?;
    // Boundary cells (offset from the vent, so a wrapped world's cells stay
    // contiguous) and a tally of the plates across them.
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut other: Vec<(u16, usize)> = Vec::new();
    for dy in -RADIUS_CELLS..=RADIUS_CELLS {
        for dx in -RADIUS_CELLS..=RADIUS_CELLS {
            if dx * dx + dy * dy > RADIUS_CELLS * RADIUS_CELLS {
                continue;
            }
            let Some(p) = at(cx + dx, cy + dy) else { continue };
            let mut edge = false;
            for (ox, oy) in [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
                if let Some(q) = at(cx + dx + ox, cy + dy + oy)
                    && q != p
                {
                    edge = true;
                    let across = if p == own { q } else { p };
                    if across != own {
                        match other.iter_mut().find(|(id, _)| *id == across) {
                            Some(e) => e.1 += 1,
                            None => other.push((across, 1)),
                        }
                    }
                }
            }
            if edge {
                pts.push((dx as f64, dy as f64));
            }
        }
    }
    if pts.is_empty() {
        return None;
    }
    if pts.len() >= MIN_CELLS {
        let n = pts.len() as f64;
        let (mx, my) = pts.iter().fold((0.0, 0.0), |(a, b), p| (a + p.0, b + p.1));
        let (mx, my) = (mx / n, my / n);
        let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
        for p in &pts {
            let (u, v) = (p.0 - mx, p.1 - my);
            sxx += u * u;
            syy += v * v;
            sxy += u * v;
        }
        // Eigenvalues of the 2x2 covariance and the major axis's angle.
        let tr = sxx + syy;
        let disc = ((sxx - syy) * (sxx - syy) + 4.0 * sxy * sxy).sqrt();
        let (l1, l2) = ((tr + disc) / 2.0, (tr - disc) / 2.0);
        if l1 > 0.0 && l1 >= MIN_ANISOTROPY * l2.max(0.0) {
            return Some(axis(0.5 * (2.0 * sxy).atan2(sxx - syy)));
        }
    }
    // The spreading direction: perpendicular to the relative velocity.
    other.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let (a, b) = (plates.get(own as usize)?, plates.get(other.first()?.0 as usize)?);
    let (rx, ry) = (b.vx - a.vx, b.vy - a.vy);
    if !(rx.is_finite() && ry.is_finite()) || rx.hypot(ry) < 1e-12 {
        return None;
    }
    Some(axis(ry.atan2(rx) + std::f64::consts::FRAC_PI_2))
}

/// An angle folded onto the axis range `0..PI`.
fn axis(a: f64) -> f64 {
    let p = std::f64::consts::PI;
    let r = a.rem_euclid(p);
    if r >= p { 0.0 } else { r }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plate(vx: f64, vy: f64) -> Plate {
        Plate { x: 0.0, y: 0.0, vx, vy, base: 0.0 }
    }

    /// Protects: the boundary's own direction. Two plates split by a line
    /// running at 30 degrees through the grid give a trend of 30 degrees
    /// (within 4 degrees: the boundary is a staircase of cells), whatever the
    /// plates' velocities; and the trend is an axis in `0..PI`.
    #[test]
    fn the_trend_follows_the_boundary_line() {
        let (gw, gh) = (120usize, 100usize);
        let a = 30f64.to_radians();
        let (nx, ny) = (-a.sin(), a.cos());
        let ids: Vec<u16> = (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f64 - 60.0, (i / gw) as f64 - 50.0);
                u16::from(x * nx + y * ny > 0.0)
            })
            .collect();
        let plates = [plate(1.0, 0.0), plate(0.0, 1.0)];
        let t = rift_trend(&ids, &plates, gw, gh, false, 60.0, 50.0).expect("a boundary");
        assert!((t - a).abs() < 4f64.to_radians(), "trend {} deg", t.to_degrees());
        assert!((0.0..std::f64::consts::PI).contains(&t));
    }

    /// Protects: the fallback and the refusal. A vent whose window holds
    /// only a tight junction of boundary cells (fewer than `MIN_CELLS`:
    /// a single cell of another plate) takes the spreading direction,
    /// perpendicular to the plates' relative velocity (plates parting east-
    /// west give a north-south trend, 90 degrees); a vent with no boundary
    /// within the radius records none.
    #[test]
    fn a_junction_takes_the_spreading_direction_and_no_boundary_none() {
        let (gw, gh) = (80usize, 80usize);
        let mut ids = vec![0u16; gw * gh];
        ids[40 * gw + 41] = 1;
        let plates = [plate(-1.0, 0.0), plate(1.0, 0.0)];
        let t = rift_trend(&ids, &plates, gw, gh, false, 40.0, 40.0).expect("a fallback");
        assert!((t - std::f64::consts::FRAC_PI_2).abs() < 1e-9, "trend {} deg", t.to_degrees());
        let flat = vec![0u16; gw * gh];
        assert_eq!(rift_trend(&flat, &plates, gw, gh, false, 40.0, 40.0), None);
    }

    /// Protects: the literal tuning (radius 24 cells, 6 cells, anisotropy 2).
    #[test]
    fn the_tuning_is_the_documented_one() {
        assert_eq!(RADIUS_CELLS, 24);
        assert_eq!(MIN_CELLS, 6);
        assert_eq!(MIN_ANISOTROPY, 2.0);
    }
}

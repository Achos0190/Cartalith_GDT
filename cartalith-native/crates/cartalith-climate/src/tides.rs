//! G3 moons & tides — `tidalForcing`/`computeTideField`/`buildTideField`/
//! `refreshTides`/`currentTideField` (reference HTML lines 5016-5048).
//!
//! Equilibrium tidal forcing from the moons (`Σ Mᵢ/dᵢ³`, `× k₂`, `× 1/g`)
//! turned into a spatial spring **tidal-range** field, amplified where
//! tides physically grow: shallow shelf seas (Green's law, `amp ∝ depth^−¼`)
//! and near coasts (funnelling). Land is exactly `0`.
//!
//! Gated the same way the geoid is: `tideField` stays `null` while
//! `planet.tides.enabled` is off, so every consumer's `tideField ? … : 0`
//! collapses.

use cartalith_jsmath::{js_exp, js_min};
use cartalith_terrain::infer::chamfer_dist;

/// One entry of `state.planet.tides.moons`. `phase` exists on the
/// reference's own moon records but `tidalForcing` never reads it, so it is
/// deliberately absent here rather than carried as a dead field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moon {
    pub mass_rel: f64,
    pub dist_rel: f64,
}

/// `tidalForcing` (reference HTML line 5022): `Σ Mᵢ/dᵢ³`.
///
/// `Math.max(0.05, m.distRel||1)` is the reference's own floor, and the
/// `||1` matters: a moon record with `distRel` missing or `0` is treated as
/// being at one Earth–Moon distance, not at zero distance. Callers building
/// a [`Moon`] from a save must apply that substitution themselves —
/// [`Moon::DEFAULT`] is the reference's own default record.
pub fn tidal_forcing(moons: &[Moon]) -> f64 {
    let mut f = 0.0;
    for m in moons {
        let d = m.dist_rel.max(0.05);
        f += m.mass_rel / (d * d * d);
    }
    f
}

impl Moon {
    /// `state.planet.tides.moons`' own default single record (reference
    /// line 12630's save-load `Object.assign`, and `currentTideField`'s own
    /// fallback at line 5043).
    pub const DEFAULT: Moon = Moon {
        mass_rel: 1.0,
        dist_rel: 1.0,
    };
}

/// `computeTideField`'s own knobs — `state.planet.tides` minus the enable
/// flag, plus `state.planet.g` (which it reads separately).
#[derive(Clone, Debug)]
pub struct TideParams {
    /// `state.planet.g`, before `computeTideField`'s own `Math.max(0.05, …)`.
    pub g: f64,
    /// Love number `k₂`; the reference's `t.k2 != null ? t.k2 : 1`.
    pub k2: f64,
    pub moons: Vec<Moon>,
}

impl Default for TideParams {
    fn default() -> Self {
        Self {
            g: 1.0,
            k2: 1.0,
            moons: vec![Moon::DEFAULT],
        }
    }
}

/// `computeTideField` (reference HTML lines 5023-5037).
///
/// `geoid` is `geoidField`: when present, sea level is local and the depth
/// every term below reads is measured against `field[i] - geoid[i]`. The
/// reference materialises that as its own `Float32Array` before use, so the
/// subtraction rounds through `f32` once and both the coast-distance
/// transform and the depth term read the *same* rounded values — reproduced
/// here rather than recomputing `field[i] - geoid[i]` at `f64` per use.
pub fn compute_tide_field(gw: usize, gh: usize, field: &[f32], geoid: Option<&[f32]>, sea: f64, p: &TideParams) -> Vec<f32> {
    let g = p.g.max(0.05);
    let a0 = 0.04 * p.k2 * tidal_forcing(&p.moons) / g;
    let n = gw * gh;
    let owned;
    let eff: &[f32] = match geoid {
        Some(gf) => {
            owned = (0..n).map(|i| (field[i] as f64 - gf[i] as f64) as f32).collect::<Vec<f32>>();
            &owned
        }
        None => field,
    };

    // `computeCoastDistance(eff, W, H, sea)` (reference line 7398) is
    // `chamferDist` (line 7423) with land as the seed mask — the two are
    // the same two-pass transform with the same `1.4142135623730951`
    // diagonal, differing only in how the seed is spelled. One
    // implementation, in `cartalith_terrain::infer`.
    let land: Vec<u8> = (0..n).map(|i| u8::from(!((eff[i] as f64) < sea))).collect();
    let cd = chamfer_dist(&land, gw, gh);
    let cscale = (gw as f64 / 40.0).max(4.0);

    let mut out = vec![0f32; n];
    for i in 0..n {
        let h = eff[i] as f64;
        if h >= sea {
            continue; // land = 0 (intertidal is flagged separately by the overlay)
        }
        let depth = sea - h;
        // Green's law: shallow water amplifies tides, capped at 3x.
        let green = js_min(3.0, (depth.max(0.02) / 0.4).powf(-0.25));
        // Funnelling/resonance near coasts (Bay-of-Fundy proxy).
        let coast = 1.0 + 1.8 * js_exp(-(cd[i] as f64) / cscale);
        out[i] = (a0 * green * coast) as f32;
    }
    out
}

/// `buildTideField`/`refreshTides` (reference HTML lines 5038-5039).
/// `None` is `tideField = null`, which is what `planet.tides.enabled`
/// defaults to.
pub fn build_tide_field(
    gw: usize,
    gh: usize,
    field: &[f32],
    geoid: Option<&[f32]>,
    sea: f64,
    enabled: bool,
    p: &TideParams,
) -> Option<Vec<f32>> {
    if !enabled {
        return None;
    }
    Some(compute_tide_field(gw, gh, field, geoid, sea, p))
}

/// `currentTideField` (reference HTML lines 5041-5048): the Tides debug
/// view previews the field **even while the toggle is off**, substituting a
/// single default moon when the roster is empty. Returns `(field, max)`;
/// the view divides by `max`.
///
/// The `1e-6` floor on the maximum is the reference's own, and it is doing
/// real work: it keeps the view's own division finite on an all-land world.
pub fn current_tide_field(
    gw: usize,
    gh: usize,
    field: &[f32],
    geoid: Option<&[f32]>,
    sea: f64,
    live: Option<&[f32]>,
    p: &TideParams,
) -> (Vec<f32>, f64) {
    let f = match live {
        Some(t) => t.to_vec(),
        None => {
            let mut cfg = p.clone();
            if cfg.moons.is_empty() {
                cfg.moons = vec![Moon::DEFAULT];
            }
            compute_tide_field(gw, gh, field, geoid, sea, &cfg)
        }
    };
    let mut mx = 1e-6f64;
    for v in f.iter() {
        if (*v as f64) > mx {
            mx = *v as f64;
        }
    }
    (f, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forcing_is_inverse_cube_and_floors_the_distance() {
        assert_eq!(tidal_forcing(&[]), 0.0);
        assert_eq!(
            tidal_forcing(&[Moon {
                mass_rel: 1.0,
                dist_rel: 1.0
            }]),
            1.0
        );
        assert_eq!(
            tidal_forcing(&[Moon {
                mass_rel: 1.0,
                dist_rel: 2.0
            }]),
            0.125
        );
        // The 0.05 floor: anything closer reads as 0.05.
        assert_eq!(
            tidal_forcing(&[Moon {
                mass_rel: 1.0,
                dist_rel: 0.01
            }]),
            tidal_forcing(&[Moon {
                mass_rel: 1.0,
                dist_rel: 0.05
            }])
        );
        // Additive across moons.
        let two = [
            Moon {
                mass_rel: 1.0,
                dist_rel: 1.0,
            },
            Moon {
                mass_rel: 0.5,
                dist_rel: 2.0,
            },
        ];
        assert_eq!(tidal_forcing(&two), 1.0 + 0.5 / 8.0);
    }

    #[test]
    fn land_is_exactly_zero_and_water_is_not() {
        let field = vec![0.9f32, 0.9, 0.1, 0.1];
        let out = compute_tide_field(2, 2, &field, None, 0.42, &TideParams::default());
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 0.0);
        assert!(out[2] > 0.0 && out[3] > 0.0);
    }

    #[test]
    fn the_disabled_gate_returns_none() {
        let field = vec![0.1f32; 4];
        assert!(build_tide_field(2, 2, &field, None, 0.42, false, &TideParams::default()).is_none());
        assert!(build_tide_field(2, 2, &field, None, 0.42, true, &TideParams::default()).is_some());
    }

    #[test]
    fn the_preview_substitutes_a_default_moon_for_an_empty_roster() {
        let field = vec![0.1f32; 4];
        let empty = TideParams {
            moons: vec![],
            ..TideParams::default()
        };
        let (f, mx) = current_tide_field(2, 2, &field, None, 0.42, None, &empty);
        assert!(mx > 1e-6, "an empty roster must still preview a real field");
        assert!(f.iter().all(|v| *v > 0.0));
        // ...and the floor holds when there genuinely is no water.
        let all_land = vec![0.9f32; 4];
        let (_, mx0) = current_tide_field(2, 2, &all_land, None, 0.42, None, &TideParams::default());
        assert_eq!(mx0, 1e-6);
    }
}

//! R5 landform classification (reference HTML lines 8075-8107).
//!
//! Morphometric classification of distinct landforms from fields the
//! simulation already produces (slope, elevation, curvature, temperature,
//! moisture, discharge) — cliffs, mesas, cirques, dunes, badlands,
//! floodplains. **First-match-wins priority**: steep forms outrank
//! climatic ones, so the branch order below is load-bearing and must not
//! be reordered into a `match`.
//!
//! Thresholds are resolution-independent because both slope and curvature
//! are scaled by `W`.

use cartalith_jsmath::js_hypot;

/// `LANDFORM_COLS` (reference line 8082) — the Landforms debug view's own
/// palette, indexed by class: 0 none, 1 cliff, 2 mesa, 3 cirque, 4 dune,
/// 5 badlands, 6 floodplain.
pub const LANDFORM_COLS: [(f64, f64, f64); 7] = [
    (30.0, 32.0, 38.0),
    (200.0, 60.0, 50.0),
    (220.0, 150.0, 60.0),
    (130.0, 190.0, 230.0),
    (235.0, 205.0, 120.0),
    (170.0, 110.0, 80.0),
    (110.0, 180.0, 110.0),
];

/// `LANDFORM_COLS`' own class names, in index order.
pub const LANDFORM_NAMES: [&str; 7] =
    ["none", "cliff", "mesa", "cirque", "dune", "badlands", "floodplain"];

/// `buildLandformField` (reference line 8083).
///
/// `temp`/`rain`/`flow` are optional exactly as in the reference, whose
/// `temp?temp[i]:15`, `rain?rain[i]:0.4` and `flow&&…` supply literal
/// fallbacks — a caller with no climate still gets cliffs and mesas.
///
/// `flow_hi` is the reference's own `riverFlowThresh(W,H)`, taken as a
/// parameter rather than recomputed: that function lives in
/// `cartalith-hydrology`, which depends on this crate, so computing it
/// here would invert the dependency. `cartalith_hydrology::river_flow_thresh`
/// is what every caller passes.
///
/// Neither edge sampling nor the 5×5 window wraps in X, matching the
/// reference — this classifier is not wrap-aware even in world mode.
pub fn build_landform_field(
    fld: &[f32],
    temp: Option<&[f32]>,
    rain: Option<&[f32]>,
    flow: Option<&[f32]>,
    w: usize,
    h: usize,
    sea: f64,
    flow_hi: f64,
) -> Vec<u8> {
    let n = w * h;
    let mut out = vec![0u8; n];
    if n == 0 {
        return out;
    }

    // Slope in height-units·W, stored through f32 the way the reference's
    // own `Float32Array sn` does -- the comparisons below read the rounded
    // value, not the f64 intermediate.
    let mut sn = vec![0f32; n];
    let at = |x: usize, y: usize| -> f64 { fld[y * w + x] as f64 };
    let neighbours = |x: usize, y: usize| -> (f64, f64, f64, f64) {
        (
            at(if x > 0 { x - 1 } else { x }, y),
            at(if x + 1 < w { x + 1 } else { x }, y),
            at(x, if y > 0 { y - 1 } else { y }),
            at(x, if y + 1 < h { y + 1 } else { y }),
        )
    };
    for y in 0..h {
        for x in 0..w {
            let (l, r, u, d) = neighbours(x, y);
            sn[y * w + x] = (js_hypot((r - l) * 0.5, (d - u) * 0.5) * w as f64) as f32;
        }
    }

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let hh = fld[i] as f64;
            if hh < sea {
                continue;
            }
            let r_norm = if (1.0 - sea) <= 0.0 { 0.0 } else { (hh - sea) / (1.0 - sea) };
            let t = temp.map_or(15.0, |a| a[i] as f64);
            let m = rain.map_or(0.4, |a| a[i] as f64);
            let (l, rr, u, d) = neighbours(x, y);
            let curv = (l + rr + u + d - 4.0 * hh) * w as f64; // Laplacian, resolution-scaled

            let mut mx = 0f64;
            for dy2 in -2i64..=2 {
                for dx2 in -2i64..=2 {
                    let xx = x as i64 + dx2;
                    let yy = y as i64 + dy2;
                    if xx < 0 || xx >= w as i64 || yy < 0 || yy >= h as i64 {
                        continue;
                    }
                    let v = sn[yy as usize * w + xx as usize] as f64;
                    if v > mx {
                        mx = v;
                    }
                }
            }

            let s = sn[i] as f64;
            // First-match-wins, in the reference's own order.
            out[i] = if s > 4.5 {
                1 // cliff: very steep face
            } else if s < 0.8 && r_norm > 0.45 && mx > 3.5 {
                2 // mesa: flat cap ringed by steep breaks
            } else if r_norm > 0.55 && curv > 0.9 && t < 2.0 {
                3 // cirque: high, cold, strongly concave hollow
            } else if t > 18.0 && m < 0.12 && s > 0.3 && s < 2.5 && r_norm < 0.5 {
                4 // dune: hot arid rolling sand country
            } else if m < 0.22 && s > 1.4 && r_norm < 0.55 && mx > 2.2 {
                5 // badlands: dry, densely dissected slopes
            } else if s < 0.5
                && r_norm < 0.35
                && flow.is_some_and(|f| {
                    (-1i64..=1).any(|dy2| {
                        (-1i64..=1).any(|dx2| {
                            let xx = x as i64 + dx2;
                            let yy = y as i64 + dy2;
                            xx >= 0
                                && xx < w as i64
                                && yy >= 0
                                && yy < h as i64
                                && f[yy as usize * w + xx as usize] as f64 > flow_hi
                        })
                    })
                })
            {
                6 // floodplain: flat low valley floor beside a trunk channel
            } else {
                0
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocean_is_never_classified() {
        let w = 6;
        let h = 6;
        let fld = vec![0.1f32; w * h];
        let out = build_landform_field(&fld, None, None, None, w, h, 0.42, 0.1);
        assert!(out.iter().all(|&v| v == 0));
    }

    /// The reference's own `temp?…:15` / `rain?…:0.4` defaults sit exactly
    /// outside the dune (`T>18`) and badlands (`M<0.22`) windows, so a
    /// no-climate call can only ever produce cliffs, mesas and floodplains.
    /// That is worth pinning: it is the difference between "the defaults
    /// are these two numbers" and "the defaults are anything at all".
    /// A single vertical step: a low bench meeting a high plateau. The two
    /// columns either side of the break clear `sn > 4.5` (cliff) and the
    /// first two plateau columns inside the 5×5 window clear `sn < 0.8`
    /// with `mx > 3.5` (mesa).
    fn step_world(w: usize, h: usize, low: f32, high: f32) -> Vec<f32> {
        (0..w * h).map(|i| if i % w < w / 2 { low } else { high }).collect()
    }

    #[test]
    fn the_no_climate_defaults_cannot_reach_the_climatic_classes() {
        let (w, h) = (24usize, 8usize);
        let fld = step_world(w, h, 0.45, 0.95);
        let out = build_landform_field(&fld, None, None, None, w, h, 0.42, 0.1);
        assert!(out.contains(&1), "the fixture must produce a cliff");
        assert!(out.contains(&2), "the fixture must produce a mesa");
        for &v in &out {
            assert!(v != 3 && v != 4 && v != 5, "class {v} is unreachable without climate");
        }
    }

    /// A cliff outranks everything below it. The *same cell* in the *same
    /// geometry* comes out badlands the moment the step is gentle enough
    /// to fall under `sn > 4.5` — which is what makes this a test of the
    /// branch order rather than of the fixture.
    #[test]
    fn first_match_wins_puts_cliff_ahead_of_badlands() {
        let (w, h) = (48usize, 6usize);
        let dry = vec![0.0f32; w * h];
        let cell = 2 * w + 23; // the low-side column at the break

        let steep = build_landform_field(&step_world(w, h, 0.43, 0.70), None, Some(&dry), None, w, h, 0.42, 0.1);
        let gentle = build_landform_field(&step_world(w, h, 0.43, 0.55), None, Some(&dry), None, w, h, 0.42, 0.1);
        assert_eq!(gentle[cell], 5, "the gentle step must satisfy the badlands predicate");
        assert_eq!(steep[cell], 1, "the same cell must come out a cliff once it also satisfies cliff");
    }

    #[test]
    fn the_palette_and_the_names_agree_on_length() {
        assert_eq!(LANDFORM_COLS.len(), LANDFORM_NAMES.len());
    }
}

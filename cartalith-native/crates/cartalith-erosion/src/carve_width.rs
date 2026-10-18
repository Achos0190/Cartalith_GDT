//! **GF-6's rock-dependent river carve** (`GEOLOGY_FIRST_SCOPE.md` §4.7, §7
//! GF-6; owner Ruling BH): the valley-width factor `ω` the carve's disc
//! half-width is multiplied by, and the knickpoints where a run leaves a
//! resistant cap for a weaker substrate.
//!
//! §4.7's formula, with `ω` the only new term:
//!
//! ```text
//! half_w = min(4·width_k, (0.8 + 0.5·(o − 1))·width_k) · ω(run)
//! ```
//!
//! `ω(run)` is the median, over the run's land points, of a per-rock factor
//! ([`valley_width_factor`]): narrower in strong rock, wider in weak.
//! Bedrock-channel width varies with rock resistance (Montgomery & Gran 2001,
//! as `GEOLOGY_FIRST_SCOPE.md` §4.7 cites it; not verified at source here).
//! The factor's values are a **judgement**.
//!
//! What this module must never change (§4.7): the carve's per-step `drop`
//! (0.0006), `CARVE_LAND_MARGIN` and `CARVE_KEEP_LAKE_DEPTH`. Those are the
//! hydrological guarantees RV-1 measured, and they live in
//! `cartalith_hydrology::carve_channel_network`, which this module does not
//! touch: it only scales the half-widths handed to it.
//!
//! The engine applies `ω` only with `WorldParams::geology_carve` on, and only
//! under the rock gate (`geology_model` and `geology_processes`). Off, the
//! half-widths are not touched, so every world without the switch is the
//! pre-GF-6 world by control flow.
//!
//! [`knickpoints`] is derived on demand from a column, a surface and the
//! channel tree, like GF-5's spring line, so it cannot disagree with the
//! column. The landmark pass reads it for *waterfall* (§4.11).

use cartalith_terrain::geology::{GeologyColumn, Rock};

/// The strength `s` at which [`valley_width_factor`] is exactly 1: §2.3's
/// moderate rocks (schist and sandstone sit at 0.60). **Judgement**: it
/// keeps the median valley on a moderate-rock world at today's width, so
/// `ω` widens weak-rock valleys and narrows strong-rock ones about a
/// neutral middle rather than shifting every valley one way.
pub const OMEGA_PIVOT_S: f64 = 0.60;

/// How much `ω` changes per unit of strength: `ω = 1 + OMEGA_SLOPE·(pivot − s)`.
/// **Judgement**: at 1.0 the factor runs from 0.75 (granite, s 0.85) to 1.50
/// (unconsolidated, s 0.10), a valley twice as wide in loose material as in
/// granite. No source found for the magnitude; Montgomery & Gran (2001) is
/// the scope's citation for the direction only.
pub const OMEGA_SLOPE: f64 = 1.0;

/// §4.7's per-rock valley-width factor, from §2.3's normalised strength `s`
/// (the one property table; no second table of widths): `1 +
/// OMEGA_SLOPE·(OMEGA_PIVOT_S − s)`. Always positive for `s` in 0..1.
pub fn valley_width_factor(rock: Rock) -> f64 {
    1.0 + OMEGA_SLOPE * (OMEGA_PIVOT_S - rock.props().s as f64)
}

/// `ω(run)`: the median of [`valley_width_factor`] over the run's **land**
/// points, reading the exposed rock on `field` (the surface the carve is
/// handed). A point's cell is the carve's own: the coordinates truncated and
/// clamped to the grid (`carve_channel_network`'s `cell`). With an even
/// count the median is the mean of the two middle values.
///
/// `None` when the run has no land point (a run lying wholly in water), and
/// then the caller leaves that run's half-width alone: there is no rock under
/// it to read, and 1.0 would be a value standing in for "no value".
pub fn run_width_factor(
    pts: &[(f64, f64)],
    column: &GeologyColumn,
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    r_expose: f32,
) -> Option<f64> {
    let mut w: Vec<f64> = pts
        .iter()
        .filter_map(|&(px, py)| {
            let x = (px as i64).clamp(0, gw as i64 - 1) as usize;
            let y = (py as i64).clamp(0, gh as i64 - 1) as usize;
            let i = y * gw + x;
            if field[i] as f64 <= sea {
                return None;
            }
            column.exposed(i, field[i], r_expose).map(valley_width_factor)
        })
        .collect();
    if w.is_empty() {
        return None;
    }
    w.sort_by(|a, b| a.partial_cmp(b).expect("valley_width_factor is finite"));
    let m = w.len() / 2;
    Some(if w.len() % 2 == 1 { w[m] } else { 0.5 * (w[m - 1] + w[m]) })
}

/// Multiplies each run's half-width by its [`run_width_factor`], in place.
/// A run with no land point keeps its half-width. Returns how many runs were
/// scaled, for the caller's measurement. `half_ws[r]` belongs to `polys[r]`.
pub fn scale_half_widths(
    polys: &[Vec<(f64, f64)>],
    half_ws: &mut [f64],
    column: &GeologyColumn,
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    r_expose: f32,
) -> usize {
    assert_eq!(polys.len(), half_ws.len(), "GF-6: one half-width per run");
    assert_eq!(field.len(), gw * gh, "GF-6: field is not gw x gh");
    let mut n = 0;
    for (pts, hw) in polys.iter().zip(half_ws.iter_mut()) {
        if let Some(w) = run_width_factor(pts, column, field, gw, gh, sea, r_expose) {
            *hw *= w;
            n += 1;
        }
    }
    n
}

/// §4.7's knickpoints: `true` on a channel cell (`chan != 0`) on land whose
/// column exposes its **cap**, and whose receiver (`recv`, the channel tree's
/// D8 receiver, `-1` for none) is land exposing its own column's
/// **substrate**, where the cap is stronger (§2.3's `s`) than that
/// substrate. That is a river leaving resistant rock for weaker rock below
/// it, where the weak rock is cut back and the cap's lip stands as a step
/// (the Niagara form). The cell marked is the lip, the last cap cell.
///
/// The strength condition is a **judgement**: §4.7 says only "where a run
/// crosses from cap to substrate", and a weak cap over strong rock gives no
/// reason for a step at the contact. A column whose two layers are the same
/// rock has no contact to cross, so it is never a knickpoint (the rule GF-5's
/// spring line uses). Must never mark a cell from a value the column does not
/// hold.
pub fn knickpoints(
    field: &[f32],
    chan: &[u8],
    recv: &[i32],
    column: &GeologyColumn,
    sea: f64,
    r_expose: f32,
) -> Vec<bool> {
    let n = field.len();
    assert!(chan.len() == n && recv.len() == n && column.len() == n, "GF-6: knickpoint inputs are not one grid");
    let land = |i: usize| field[i] as f64 > sea;
    // The cap this cell exposes, if it is a two-layer column exposing its cap.
    let cap = |i: usize| -> Option<Rock> {
        let (top, (sub, _)) = (column.top(i)?, column.substrate(i)?);
        (top != sub && column.exposed(i, field[i], r_expose) == Some(top)).then_some(top)
    };
    // The substrate this cell exposes, if it is a two-layer column exposing it.
    let substrate = |i: usize| -> Option<Rock> {
        let (top, (sub, _)) = (column.top(i)?, column.substrate(i)?);
        (top != sub && column.exposed(i, field[i], r_expose) == Some(sub)).then_some(sub)
    };
    (0..n)
        .map(|i| {
            if chan[i] == 0 || !land(i) {
                return false;
            }
            let r = recv[i];
            if r < 0 || r as usize >= n || !land(r as usize) {
                return false;
            }
            match (cap(i), substrate(r as usize)) {
                (Some(c), Some(s)) => c.props().s > s.props().s,
                _ => false,
            }
        })
        .collect()
}

/// Pins `ω` and the knickpoint rule on hand-built columns against literal
/// values, never against this module's own constants.
#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_terrain::geology::{Construction, NO_LAYER};

    /// A column of cells, each `top` over an optional `(sub, contact)`, bare
    /// rock (no regolith).
    fn column(tops: &[Rock], subs: &[Option<(Rock, f32)>]) -> GeologyColumn {
        GeologyColumn {
            rock_top: tops.iter().map(|&r| r as u8).collect(),
            rock_sub: subs.iter().map(|s| s.map_or(NO_LAYER, |(r, _)| r as u8)).collect(),
            contact: subs.iter().map(|s| s.map_or(f32::NAN, |(_, c)| c)).collect(),
            regolith: vec![0.0; tops.len()],
            volcanic_setting: vec![0; tops.len()],
            construction: Construction::Off,
        }
    }

    /// Protects §4.7's direction and the judged magnitude: every rock's
    /// factor as a literal, so a change to the pivot, the slope or a rock's
    /// `s` goes red here.
    #[test]
    fn the_valley_width_factor_of_every_rock_is_pinned() {
        let want = [
            (Rock::Granite, 0.75),
            (Rock::Gneiss, 0.80),
            (Rock::Schist, 1.00),
            (Rock::PlateauBasalt, 0.85),
            (Rock::OceanicBasalt, 0.85),
            (Rock::Andesite, 0.90),
            (Rock::Tuff, 1.25),
            (Rock::Limestone, 0.90),
            (Rock::Sandstone, 1.00),
            (Rock::Shale, 1.30),
            (Rock::Unconsolidated, 1.50),
        ];
        assert_eq!(want.len(), Rock::ALL.len(), "every rock is pinned");
        for (r, w) in want {
            assert!((valley_width_factor(r) - w).abs() < 1e-6, "{r:?}: {} != {w}", valley_width_factor(r));
        }
    }

    /// Protects the median, the land filter and the "no land, no factor"
    /// refusal, on a 4 x 1 strip at sea 0.5.
    #[test]
    fn the_run_factor_is_the_median_over_land_points() {
        let col = column(&[Rock::Granite, Rock::Shale, Rock::Shale, Rock::Unconsolidated], &[None; 4]);
        let field = [0.6f32, 0.6, 0.6, 0.4];
        let pt = |x: f64| (x + 0.5, 0.5);
        // Odd count: granite 0.75, shale 1.30, shale 1.30 -> 1.30.
        let w = run_width_factor(&[pt(0.0), pt(1.0), pt(2.0)], &col, &field, 4, 1, 0.5, 0.0).unwrap();
        assert!((w - 1.30).abs() < 1e-6, "odd median {w}");
        // Even count: granite 0.75, shale 1.30 -> 1.025.
        let w = run_width_factor(&[pt(0.0), pt(1.0)], &col, &field, 4, 1, 0.5, 0.0).unwrap();
        assert!((w - 1.025).abs() < 1e-6, "even median {w}");
        // The water point (cell 3, unconsolidated, 1.50) is not read.
        let w = run_width_factor(&[pt(0.0), pt(3.0)], &col, &field, 4, 1, 0.5, 0.0).unwrap();
        assert!((w - 0.75).abs() < 1e-6, "a water point counted: {w}");
        assert_eq!(run_width_factor(&[pt(3.0)], &col, &field, 4, 1, 0.5, 0.0), None, "a run wholly in water has no factor");
    }

    /// Protects the in-place scaling and that an all-water run keeps its width.
    #[test]
    fn scaling_multiplies_each_run_and_skips_an_all_water_run() {
        let col = column(&[Rock::Granite, Rock::Shale, Rock::Shale, Rock::Unconsolidated], &[None; 4]);
        let field = [0.6f32, 0.6, 0.6, 0.4];
        let polys = vec![vec![(0.5, 0.5)], vec![(1.5, 0.5)], vec![(3.5, 0.5)]];
        let mut hw = vec![2.0, 2.0, 2.0];
        let n = scale_half_widths(&polys, &mut hw, &col, &field, 4, 1, 0.5, 0.0);
        assert_eq!(n, 2);
        assert!((hw[0] - 1.5).abs() < 1e-6 && (hw[1] - 2.6).abs() < 1e-6 && hw[2] == 2.0, "{hw:?}");
    }

    /// Protects every knickpoint condition: the lip is marked; a weaker cap,
    /// a same-rock column, a cap-to-cap step, a non-channel cell, a missing
    /// receiver and a receiver in the sea each are not.
    #[test]
    fn a_knickpoint_is_a_channel_leaving_a_stronger_cap_for_its_substrate() {
        // 1 x 6 strip, sea 0.3. Each case is a pair (i, recv[i]).
        // Contact 0.5: a surface at 0.6 exposes the cap, at 0.4 the substrate.
        let lime_shale = Some((Rock::Shale, 0.5f32));
        let shale_granite = Some((Rock::Granite, 0.5f32));
        let tops = [Rock::Limestone, Rock::Limestone, Rock::Shale, Rock::Shale, Rock::Limestone, Rock::Limestone];
        let subs = [lime_shale, lime_shale, shale_granite, shale_granite, lime_shale, lime_shale];
        let col = column(&tops, &subs);
        let field = [0.6f32, 0.4, 0.6, 0.4, 0.6, 0.6];
        let chan = [1u8, 1, 1, 1, 1, 1];
        let recv = [1i32, -1, 3, -1, 5, -1];
        let k = knickpoints(&field, &chan, &recv, &col, 0.3, 0.0);
        // limestone (s 0.70) over shale (0.30): cell 0 is the lip.
        assert!(k[0], "limestone cap onto shale substrate");
        // shale (0.30) over granite (0.85): a weaker cap, no step.
        assert!(!k[2], "a weaker cap is not a knickpoint");
        // cap onto cap: no contact crossed.
        assert!(!k[4], "cap to cap");
        // ends of the chains: no receiver.
        assert!(!k[1] && !k[3] && !k[5]);
        // The same pair with the channel removed.
        let k = knickpoints(&field, &[0, 1, 1, 1, 1, 1], &recv, &col, 0.3, 0.0);
        assert!(!k[0], "not a channel");
        // The receiver below sea.
        let k = knickpoints(&[0.6, 0.2, 0.6, 0.4, 0.6, 0.6], &chan, &recv, &col, 0.3, 0.0);
        assert!(!k[0], "receiver in the sea");
        // A same-rock two-layer column has no contact.
        let same = column(&[Rock::Limestone, Rock::Limestone], &[Some((Rock::Limestone, 0.5)), Some((Rock::Limestone, 0.5))]);
        let k = knickpoints(&[0.6, 0.4], &[1, 1], &[1, -1], &same, 0.3, 0.0);
        assert!(!k[0], "same rock both layers");
        // A receiver whose own column is shale over shale, below its contact:
        // it exposes no distinct substrate, so the cap above it is no lip.
        let col = column(&[Rock::Limestone, Rock::Shale], &[Some((Rock::Shale, 0.5)), Some((Rock::Shale, 0.5))]);
        let k = knickpoints(&[0.6, 0.4], &[1, 1], &[1, -1], &col, 0.3, 0.0);
        assert!(!k[0], "a same-rock receiver column");
    }
}

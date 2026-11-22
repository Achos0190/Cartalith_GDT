//! The ice/white-area vertical streaks (`OUTSTANDING_WORK.md`, "Ice/white map streaks"; owner frames on
//! seed 4242, 256 x 192, 2 400 km): the map's snow showed a column-wise ripple. The height field read in the
//! app had alternate columns 150 m apart at the top of a high plateau, so the streaks were in the generated
//! terrain, not the renderer (`_small3_probe.gd --mode dump`; 20 render-term ablations did not remove them).
//! `ripple_ablation` (ignored) is the generation ablation that found it: `orogen_relief` off removes the
//! alternation (0.641 along x against 1.374), every other switch leaves it. Cause and fix are
//! `cartalith_terrain::orogen::MIN_LATTICE_CELLS`; the literal regression bar is that crate's
//! `a_coarse_grid_gets_no_nyquist_gullies_and_a_fine_one_is_unchanged`.
//!
//! `alternation` is the measurement: over the cells whose 3x3 neighbourhood is all above `HIGH`, the mean absolute
//! second difference over the mean absolute first difference, along x and along y. A smooth field reads well
//! under 1, a Nyquist alternation near 2. Run: `cargo test -p cartalith-godot --test ice_column_ripple --
//! --ignored --nocapture` (`RIPPLE_ARMS=head,no_orogen` filters).
#![allow(dead_code)]
#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;
use cartalith_engine::{generate_terrain, WorldParams};

/// A cell is "high" above this height-field value (sea level is 0.42; the streaks sit near the top of the range).
const HIGH: f32 = 0.62;

fn app(seed: i32) -> WorldParams {
    let mut p = params::defaults();
    p.gw = 256;
    p.gh = 192;
    p.tect.seed = seed;
    p.map_width_km = 2400.0;
    p.sea_level = 0.42;
    cartalith_engine::resolve_peak(&mut p);
    p
}

/// The grid-scale alternation of the high ground: mean |second difference| / mean |first difference| along x
/// and along y, over the cells whose 3x3 neighbourhood is all above `HIGH`. A smooth field reads well under 1
/// (the second difference of a slowly changing slope is small against the slope); alternate columns read near 2.
/// Returns `(along_x, along_y, cells)`.
fn alternation(f: &[f32], gw: usize, gh: usize) -> (f64, f64, usize) {
    let (mut d2x, mut d1x, mut d2y, mut d1y, mut n) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let at = |dx: isize, dy: isize| f[(y as isize + dy) as usize * gw + (x as isize + dx) as usize] as f64;
            if (-1..=1).all(|dy| (-1..=1).all(|dx| at(dx, dy) > HIGH as f64)) {
                d2x += (at(-1, 0) - 2.0 * at(0, 0) + at(1, 0)).abs();
                d1x += (at(1, 0) - at(0, 0)).abs();
                d2y += (at(0, -1) - 2.0 * at(0, 0) + at(0, 1)).abs();
                d1y += (at(0, 1) - at(0, 0)).abs();
                n += 1;
            }
        }
    }
    (d2x / d1x.max(1e-12), d2y / d1y.max(1e-12), n)
}

type Edit = fn(&mut WorldParams);
const ARMS: [(&str, Edit); 9] = [
    ("head", |_| {}),
    ("carve_off", |p| p.carve_rivers = false),
    ("no_glacial", |p| p.passes.glacial = false),
    ("no_velocity", |p| p.passes.velocity = false),
    ("no_hillslope", |p| p.passes.hillslope = false),
    ("no_coastal", |p| p.passes.coastal = false),
    ("no_orogen", |p| p.tect.orogen_relief = false),
    ("no_ridged", |p| p.tect.ridged = false),
    ("no_volc_edifice", |p| p.volc.edifice_model = false),
];

#[test]
#[ignore = "measurement: generates a world per arm"]
fn ripple_ablation() {
    let only = std::env::var("RIPPLE_ARMS").ok();
    for (name, edit) in ARMS.iter().filter(|(n, _)| only.as_deref().is_none_or(|o| o.split(',').any(|t| t == *n))) {
        let mut p = app(4242);
        edit(&mut p);
        let ws = generate_terrain(&p);
        let (ax, ay, n) = alternation(&ws.field, p.gw, p.gh);
        println!("RIPPLE {name:<14} along_x={ax:.3} along_y={ay:.3} cells={n}");
        let row: Vec<String> = (170..200).map(|x| format!("{:.0}", (ws.field[80 * p.gw + x] as f64 - 0.42) / 0.58 * p.peak_m)).collect();
        println!("  row80 {}", row.join(" "));
    }
}

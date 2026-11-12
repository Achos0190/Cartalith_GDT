//! **RR-6: which lakes a water balance would make appear or vanish -- measured,
//! no switch flipped** (`RIVER_REALISM_SCOPE.md` §RR-6; owner Ruling ES item
//! 4, 2026-11-12: *"Measure first"* -- "Build RR-6's harness only: report the
//! lakes that would appear or vanish on the owner world and two seeds, with a
//! frame. Flip no switch.").
//!
//! Protects: the harness's report of what the shipped balance does. Since
//! Ruling ET (2026-11-12) the balance is `cartalith_civ::lake_balance`, called
//! by the generation path behind `WorldParams::lake_balance` and by this file
//! (one implementation: [`lake_pools`]); this file only classifies each pool's
//! change. The assertions guard the harness's own validity: lakes exist on every world,
//! the do-nothing control (zero evaporation) keeps every pool full, and the
//! table is not silently empty.
//!
//! **Today** (`cartalith_civ::build_water_bodies`): an above-sea cell is lake
//! when it lies in a filled depression more than `LAKE_DEPTH` deep **and** its
//! own rain is at least `LAKE_RAIN`; every lake stands at its spill height.
//!
//! **The balance** (`LAKE_FILL_RESEARCH.md` Part 1(b), carried in the scope
//! §2.4: a closed lake's area adjusts until evaporation over it balances its
//! inflow; level `= min(spill, balance)`). The unit is a **pool**: a 4-connected
//! component of above-sea cells deeper than `LAKE_DEPTH` below the fill level,
//! whatever their rain (the balance replaces the per-cell rain test). Per pool:
//! * inflow `I` = the routed discharge (`flow_discharge`, rain-weighted
//!   cell-units, mean 1 per cell) at its highest-discharge cell -- the water
//!   the river network brings to it, rain on the pool included (the scope's
//!   bar: "a basin's inflow must be the routed discharge at its cells, not a
//!   guess");
//! * evaporation per pool cell `e(T) = EVAP_AT_25 * clamp(T, 0, T_MAX) / 25`,
//!   in the same cell-units (a Thornthwaite-shaped, linear-in-temperature
//!   proxy; **labelled judgement**, see [`EVAP_AT_25`]);
//! * the pool's cells sorted by height; the balanced lake is the lowest `k`
//!   cells with `sum(e) <= I` (the level falls until the shrinking area's
//!   evaporation matches the inflow); `k` = all cells means it spills and keeps
//!   its outlet; a balanced lake shallower than `LAKE_DEPTH` is a **playa**.
//!
//! What it reports per world: pools; lakes today; and per pool the change
//! (vanish = a lake today, a playa under the balance; appear = no lake today
//! because its cells are dry, a lake under the balance because a river feeds
//! it; shrink / grow by more than 10 % of cells; loses its outlet = a lake
//! today that no longer spills), the largest of each listed with its centre,
//! areas, `I` and `E` at spill; and a PNG frame per world (teal both, red
//! vanishes or shrinks away, green appears or grows, dark blue ocean).
//!
//! ```text
//! cargo test --release -p cartalith-godot --test lake_balance_harness -- --ignored --nocapture --test-threads=1
//! ```
//! `LAKEBAL_DIR` chooses where the frames go (default: `lakebal/` under the
//! system temp dir).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_civ::lake_balance::{EVAP_AT_25, T_MAX, lake_pools};
use cartalith_civ::{LAKE_DEPTH, LAKE_RAIN, build_water_bodies};
use cartalith_engine::generate_terrain;

/// A change smaller than this share of a pool's cells is "same".
const SAME_SHARE: f64 = 0.10;

const WORLDS: [(i32, usize, usize, f64, f64, &str); 3] = [
    (246_371, 2048, 1311, 40_075.0, 1.55, "owner"),
    (483_920, 1024, 656, 1_200.0, 1.0, "483920"),
    (912_345, 1024, 656, 1_200.0, 1.0, "912345"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Change {
    Same,
    Vanish,
    Appear,
    Shrink,
    Grow,
}

struct Pool {
    cells: Vec<usize>,
    today: usize,
    balanced: Vec<usize>,
    inflow: f64,
    e_spill: f64,
    spills: bool,
    change: Change,
}

/// The pools of a world and each one's change against today's lakes, at
/// evaporation scale `ek`. The balance itself is `cartalith_civ::lake_balance`.
fn pools(field: &[f32], flow: &[f32], temp: &[f32], class: &[u8], fill: &[f32], gw: usize, gh: usize, sea: f64, wrap: bool, ek: f64) -> Vec<Pool> {
    lake_pools(field, flow, temp, fill, gw, gh, sea, wrap, ek)
        .into_iter()
        .map(|q| {
            let today = q.cells.iter().filter(|&&i| class[i] == 2).count();
            let b = q.balanced.len();
            let change = if today > 0 && b == 0 {
                Change::Vanish
            } else if today == 0 && b > 0 {
                Change::Appear
            } else if (b as f64) < today as f64 * (1.0 - SAME_SHARE) {
                Change::Shrink
            } else if (b as f64) > today as f64 * (1.0 + SAME_SHARE) && today > 0 {
                Change::Grow
            } else {
                Change::Same
            };
            Pool { cells: q.cells, today, balanced: q.balanced, inflow: q.inflow, e_spill: q.e_spill, spills: q.spills, change }
        })
        .collect()
}

fn centre(cells: &[usize], gw: usize) -> (usize, usize) {
    let (sx, sy) = cells.iter().fold((0usize, 0usize), |(a, b), &i| (a + i % gw, b + i / gw));
    (sx / cells.len().max(1), sy / cells.len().max(1))
}

#[test]
#[ignore]
fn lake_balance_measurement() {
    let out = std::env::var("LAKEBAL_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir().join("lakebal"));
    std::fs::create_dir_all(&out).expect("create output dir");
    println!("lake balance (RR-6): LAKE_DEPTH {LAKE_DEPTH}, LAKE_RAIN {LAKE_RAIN} (today's gate); e(T) = {EVAP_AT_25} * clamp(T, 0, {T_MAX}) / 25 cell-units per lake cell; same within {:.0} %", SAME_SHARE * 100.0);
    for &(seed, gw, gh, km, density, label) in WORLDS.iter() {
        let mut p = params::defaults();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = km;
        p.river_density = density;
        p.use_gpu = false;
        let ws = generate_terrain(&p);
        let wrap = p.world;
        let sea = ws.sea_level;
        let wb = build_water_bodies(&ws.field, gw, gh, sea, wrap, Some(&ws.rainfall));
        let control = pools(&ws.field, &ws.flow_discharge, &ws.temperature, &wb.classification, &wb.fill_level, gw, gh, sea, wrap, 0.0);
        assert!(control.iter().all(|q| q.spills), "{label}: with no evaporation every pool must spill (the do-nothing control)");
        let ps = pools(&ws.field, &ws.flow_discharge, &ws.temperature, &wb.classification, &wb.fill_level, gw, gh, sea, wrap, 1.0);
        let lakes_today = ps.iter().filter(|q| q.today > 0).count();
        assert!(lakes_today > 0, "{label}: no above-sea lake today -- the fixture did not reach the code");
        let count = |c: Change| ps.iter().filter(|q| q.change == c).count();
        let cells_today: usize = ps.iter().map(|q| q.today).sum();
        let cells_bal: usize = ps.iter().map(|q| q.balanced.len()).sum();
        let lose_outlet = ps.iter().filter(|q| q.today > 0 && !q.spills).count();
        let lakes_bal = ps.iter().filter(|q| !q.balanced.is_empty()).count();
        println!(
            "\n=== {label}: seed {seed}, {gw}x{gh}, {km} km, density {density}: pools {}; lakes today {lakes_today} ({cells_today} cells), under the balance {lakes_bal} ({cells_bal} cells, {:+.0} %)",
            ps.len(),
            (cells_bal as f64 / cells_today.max(1) as f64 - 1.0) * 100.0
        );
        println!(
            "  vanish {} | appear {} | shrink {} | grow {} | same {} | lakes today that would lose their outlet {lose_outlet}",
            count(Change::Vanish),
            count(Change::Appear),
            count(Change::Shrink),
            count(Change::Grow),
            count(Change::Same)
        );
        for c in [Change::Vanish, Change::Appear, Change::Shrink, Change::Grow] {
            let mut v: Vec<&Pool> = ps.iter().filter(|q| q.change == c).collect();
            v.sort_by_key(|q| std::cmp::Reverse(q.today.max(q.balanced.len())));
            for q in v.iter().take(8) {
                let (cx, cy) = centre(&q.cells, gw);
                println!(
                    "    {c:?}: cell ({cx}, {cy}), pool {} cells, lake today {} -> balance {}, inflow {:.1}, evaporation at spill {:.1}, spills {}",
                    q.cells.len(),
                    q.today,
                    q.balanced.len(),
                    q.inflow,
                    q.e_spill,
                    q.spills
                );
            }
        }
        // The frame: 1 px a cell.
        let top = ws.field.iter().copied().fold(f32::MIN, f32::max) as f64;
        let mut rgb = vec![0u8; gw * gh * 3];
        let mut bal = vec![false; gw * gh];
        for q in &ps {
            for &i in &q.balanced {
                bal[i] = true;
            }
        }
        for i in 0..gw * gh {
            let now = wb.classification[i] == 2 && ws.field[i] as f64 > sea;
            let px: [u8; 3] = if wb.classification[i] == 1 || (wb.classification[i] == 2 && ws.field[i] as f64 <= sea) {
                [28, 48, 82]
            } else if now && bal[i] {
                [0, 200, 190]
            } else if now {
                [255, 60, 30]
            } else if bal[i] {
                [80, 255, 60]
            } else {
                let t = ((ws.field[i] as f64 - sea) / (top - sea)).clamp(0.0, 1.0);
                let g = (90.0 + 150.0 * t) as u8;
                [g, g, g]
            };
            rgb[i * 3..i * 3 + 3].copy_from_slice(&px);
        }
        let path = out.join(format!("lake_balance_{label}.png"));
        let file = std::fs::File::create(&path).expect("create png");
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), gw as u32, gh as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().expect("png header").write_image_data(&rgb).expect("png data");
        println!("  frame {}", path.display());
    }
}

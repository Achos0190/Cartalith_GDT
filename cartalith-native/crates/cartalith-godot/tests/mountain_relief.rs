//! **Ruling CU item 1** (`LARGE_ITEM_RULINGS.md`, owner, 2026-10-19) -- the
//! height-field statistics of `MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md`
//! §1/§2.1 ("hstats"), rebuilt in Rust over a real generated world under the
//! app's own generation parameters (`params::defaults()`), so the mountain
//! work can be measured before and after on the investigation's nine worlds
//! without a display.
//!
//! What it measures, per world (the investigation's definitions, §1
//! "Statistics"): land is `field >= sea_level` (the world's own, re-anchored
//! sea level on archetype worlds); elevation is `metersPerUnit`'s anchoring
//! (`1 - sea` maps to `peak_m`); "top" is the top decile of land **by height**
//! (an input, never a structure measure -- `MISTAKES.md`, "Measure an effect
//! inside a subset of pixels") against the rest of the land; slope is the
//! central-difference gradient in real metres per real metre; a crest (valley)
//! cell is a strict profile maximum (minimum) along at least 2 of the 4 axes;
//! the channel fraction is D8 accumulation of at least 32 cells
//! (`cartalith_hydrology::compute_flow`, no rain). It adds the top decile's
//! Laplacian standard deviation (metres), which the investigation quoted for
//! its strong-erosion arm.
//!
//! It also counts what derives from height, for the regression check the
//! ruling asks for: river cells (`river_mask`), traced river polylines,
//! drainage basins reaching the sea, land fraction, and the land-mask
//! agreement with an `MR_BASE` reference run when one is given.
//!
//! Since the 2026-10-19 follow-up it also reports the flat-fill patches
//! (`flat_patches`: the defect "filled hollows draw as flat basins") and an
//! FNV-1a hash of the final field (`field`), the identity check for a
//! switch-off arm against an earlier build.
//!
//! One `HSTAT` line per world; with `MR_OUT` set, a plain hillshade of the
//! final heights with the flat cells drawn red (not the product renderer --
//! the investigation's shading-independent check) as
//! `<MR_OUT>/<tag><MR_TAG>_hill.png`, and the
//! raw land mask as `<tag><MR_TAG>_land.bin` for `MR_BASE` comparisons.
//! `#[ignore]`d: it generates up to nine 2048 x 1311 worlds.
//!
//! ```text
//! MR_OUT=<dir> MR_TAG=_base cargo test -p cartalith-godot --test mountain_relief -- --ignored --nocapture
//! ```
//!
//! Environment: `MR_WORLDS` (tag prefixes), `MR_GW`/`MR_GH`, `MR_OUT`,
//! `MR_TAG`, `MR_BASE` (a tag whose `_land.bin` files to compare against),
//! `MR_SWITCH` (a comma list of generation edits; an unknown token is refused).
//!
//! Protects: nothing by itself (a measurement harness, `#[ignore]`d); it
//! asserts only that each world has land and a non-empty top decile.

#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

/// One world of the investigation's nine (§1, "Worlds").
struct WorldSpec {
    tag: &'static str,
    seed: i32,
    /// An archetype's knobs, or `None` for Classic.
    archetype: Option<[f64; 5]>,
    width_km: f64,
}

/// `lib.rs`'s `ARCHETYPES` rows, quoted (the table is private to the cdylib).
const EARTH: [f64; 5] = [0.30, 0.50, 0.60, 0.60, 0.20];
const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];
const SUPERCONTINENT: [f64; 5] = [0.60, 0.10, 0.50, 0.70, 0.10];

/// The investigation's nine worlds, in its §2.1 table order.
const WORLDS: [WorldSpec; 9] = [
    WorldSpec { tag: "s314159_earth_800", seed: 314159, archetype: Some(EARTH), width_km: 800.0 },
    WorldSpec { tag: "s24601_archipelago_800", seed: 24601, archetype: Some(ARCHIPELAGO), width_km: 800.0 },
    WorldSpec { tag: "s24601_earth_3000", seed: 24601, archetype: Some(EARTH), width_km: 3000.0 },
    WorldSpec { tag: "s483920_classic_800", seed: 483920, archetype: None, width_km: 800.0 },
    WorldSpec { tag: "s71077345_classic_800", seed: 71077345, archetype: None, width_km: 800.0 },
    WorldSpec { tag: "s246371_earth_800", seed: 246371, archetype: Some(EARTH), width_km: 800.0 },
    WorldSpec { tag: "s24601_classic_800", seed: 24601, archetype: None, width_km: 800.0 },
    WorldSpec { tag: "s314159_classic_3000", seed: 314159, archetype: None, width_km: 3000.0 },
    WorldSpec { tag: "s483920_supercontinent_3000", seed: 483920, archetype: Some(SUPERCONTINENT), width_km: 3000.0 },
];

/// The app's parameters for one world, as `zoom_detail_ladder.rs`' own
/// `world_params` builds them (sea level 0.42, the investigation's).
fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = w.width_km;
    p.sea_level = 0.42;
    if let Some(k) = w.archetype {
        p.world_structure = cartalith_engine::WorldStructureParams {
            enabled: true,
            continentality: k[0],
            fragmentation: k[1],
            tectonic_energy: k[2],
            ocean_depth: k[3],
            hotspot_density: k[4],
        };
    }
    if let Ok(v) = std::env::var("MR_SWITCH") {
        for t in v.split(',').filter(|t| !t.is_empty()) {
            match t {
                "carve_off" => p.carve_rivers = false,
                // The app before Ruling CU item 1: the orogen relief term off.
                "orogen_off" => p.tect.orogen_relief = false,
                other => panic!("MR_SWITCH: unknown switch {other}"),
            }
        }
    }
    p
}

/// Percentile `q` (0..1) of `v` by nearest rank on a sorted copy.
fn pct(v: &[f64], q: f64) -> f64 {
    assert!(!v.is_empty(), "percentile of an empty set");
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() - 1) as f64 * q).round() as usize]
}

/// Population standard deviation.
fn std(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt()
}

/// The statistics of one finished world (see the module doc).
fn hstats(ws: &cartalith_engine::WorldState, p: &cartalith_engine::WorldParams) -> String {
    let (gw, gh) = (p.gw, p.gh);
    let f = &ws.field;
    let sea = ws.sea_level;
    let denom = if 1.0 - sea == 0.0 { 1e-6 } else { 1.0 - sea };
    let em = |i: usize| (f[i] as f64 - sea) / denom * p.peak_m;
    let cell_m = p.map_width_km * 1000.0 / gw as f64;
    let land: Vec<usize> = (0..gw * gh).filter(|&i| f[i] as f64 >= sea).collect();
    assert!(!land.is_empty(), "a world with no land");
    let elev: Vec<f64> = land.iter().map(|&i| em(i)).collect();
    let p90 = pct(&elev, 0.90);
    let p99 = pct(&elev, 0.99);
    // Interior cells only for the neighbourhood measures (no wrap: the
    // investigation's export was a plain raster).
    let inside = |i: usize| {
        let (x, y) = (i % gw, i / gw);
        x >= 1 && y >= 1 && x + 1 < gw && y + 1 < gh
    };
    let flow = cartalith_hydrology::compute_flow(gw, gh, f, None, false, p.world);
    let (mut top_s, mut rest_s, mut top_lap) = (Vec::new(), Vec::new(), Vec::new());
    let (mut tc, mut rc, mut tv, mut rv, mut tch, mut rch, mut tn, mut rn) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut tp, mut rp) = (0usize, 0usize);
    for &i in &land {
        if !inside(i) {
            continue;
        }
        let h = em(i);
        let gx = (em(i + 1) - em(i - 1)) / (2.0 * cell_m);
        let gy = (em(i + gw) - em(i - gw)) / (2.0 * cell_m);
        let slope = gx.hypot(gy).atan().to_degrees();
        // The four axes: E-W, N-S and the two diagonals.
        let axes = [(1usize, 1usize), (gw, gw), (gw + 1, gw + 1), (gw - 1, gw - 1)];
        let mut crest = 0;
        let mut valley = 0;
        for (a, b) in axes {
            let (lo, hi) = (em(i - a), em(i + b));
            if h > lo && h > hi {
                crest += 1;
            }
            if h < lo && h < hi {
                valley += 1;
            }
        }
        // A pit: strictly below all eight neighbours (a closed depression a
        // plain D8 walk stops in).
        let pit = [1usize, gw, gw + 1, gw - 1].iter().all(|&a| h < em(i - a) && h < em(i + a));
        let lap = em(i + 1) + em(i - 1) + em(i + gw) + em(i - gw) - 4.0 * h;
        let is_top = h >= p90;
        let ch = flow[i] >= 32.0;
        if is_top {
            tn += 1;
            top_s.push(slope);
            top_lap.push(lap);
            tc += (crest >= 2) as usize;
            tv += (valley >= 2) as usize;
            tch += ch as usize;
            tp += pit as usize;
        } else {
            rn += 1;
            rest_s.push(slope);
            rc += (crest >= 2) as usize;
            rv += (valley >= 2) as usize;
            rch += ch as usize;
            rp += pit as usize;
        }
    }
    assert!(tn > 0 && rn > 0, "an empty top decile or rest");
    let gt30 = top_s.iter().filter(|&&s| s > 30.0).count() as f64 / tn as f64;
    // Flat ground in the top decile (slope under 0.5°): filled basins read here.
    let flat = top_s.iter().filter(|&&s| s < 0.5).count() as f64 / tn as f64;
    let clipped = land.iter().filter(|&&i| f[i] >= 1.0).count() as f64 / land.len() as f64;
    // Regression counts: what derives from height.
    let rivers = ws.river_mask.as_ref().map_or(0, |m| m.iter().filter(|&&v| v > 0).count());
    // On a classic world `compute_height`'s uplift term is the stress field
    // itself, so the orogen belt weight can be rebuilt from an INPUT (never
    // from the field under test): channel and crest fractions inside the belt
    // (weight >= 0.5) against the land outside it. Absent on archetype worlds,
    // whose orogeny field `WorldState` does not keep.
    let belt = if p.world_structure.enabled {
        " | belt -".to_string()
    } else {
        let tn = cartalith_terrain::orogen::OrogenTuning::default();
        let r = (tn.belt_km / (p.map_width_km / gw as f64)).round() as i64;
        let m = cartalith_terrain::orogen::belt_weight(&ws.stress_field, gw, gh, r, p.world, &tn);
        let (mut bn, mut bch, mut on, mut och) = (0usize, 0usize, 0usize, 0usize);
        for &i in &land {
            if !inside(i) {
                continue;
            }
            let ch = (flow[i] >= 32.0) as usize;
            if m[i] >= 0.5 {
                bn += 1;
                bch += ch;
            } else {
                on += 1;
                och += ch;
            }
        }
        format!(" | belt n {} chan {:.4}/{:.4}", bn, bch as f64 / bn.max(1) as f64, och as f64 / on.max(1) as f64)
    };
    let chans = ws.channels.as_ref().map_or(0, |c| c.chan.iter().filter(|&&v| v > 0).count());
    let (fl, _) = flat_patches(f, gw, gh, sea, denom, p.peak_m);
    format!(
        "land {:.3} p90 {:.0} p99 {:.0} | top slope p50 {:.2} p90 {:.2} gt30 {:.3} flat {:.3} | rest slope p50 {:.2} | crest {:.4}/{:.4} valley {:.4}/{:.4} chan {:.4}/{:.4} | pits {:.4}/{:.4} | lapstd {:.1} | clipped {:.4} | rivers {} chans {}{} | {}",
        land.len() as f64 / (gw * gh) as f64,
        p90,
        p99,
        pct(&top_s, 0.5),
        pct(&top_s, 0.9),
        gt30,
        flat,
        pct(&rest_s, 0.5),
        tc as f64 / tn as f64,
        rc as f64 / rn as f64,
        tv as f64 / tn as f64,
        rv as f64 / rn as f64,
        tch as f64 / tn as f64,
        rch as f64 / rn as f64,
        tp as f64 / tn as f64,
        rp as f64 / rn as f64,
        std(&top_lap),
        clipped,
        rivers,
        chans,
        belt,
        fl,
    )
}

/// **Flat-fill patches** (the 2026-10-19 follow-up's defect (a), "filled belt
/// hollows draw as flat basins"): a Priority-Flood+ε fill leaves its filled
/// cells one `f32` step apart, so a filled hollow is a patch of land whose
/// four neighbours all lie within [`FLAT_EPS_M`] metres of it. Natural ground
/// on this grid is never that flat away from the coast, so the land below
/// [`FLAT_MIN_M`] metres (deltas, sea-anchored plains) is excluded. Reports the
/// share of that land in flat cells, the number of 4-connected flat patches of
/// at least 16 cells, and the share of the land in those patches.
fn flat_patches(f: &[f32], gw: usize, gh: usize, sea: f64, denom: f64, peak_m: f64) -> (String, Vec<bool>) {
    let em = |i: usize| (f[i] as f64 - sea) / denom * peak_m;
    let n = gw * gh;
    let mut flat = vec![false; n];
    let mut land = 0usize;
    let mut nflat = 0usize;
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let i = y * gw + x;
            let h = em(i);
            if h < FLAT_MIN_M {
                continue;
            }
            land += 1;
            if [i - 1, i + 1, i - gw, i + gw].iter().all(|&j| (em(j) - h).abs() < FLAT_EPS_M) {
                flat[i] = true;
                nflat += 1;
            }
        }
    }
    assert!(land > 0, "no land above {FLAT_MIN_M} m");
    let mut seen = vec![false; n];
    let (mut patches, mut in_patches) = (0usize, 0usize);
    let mut stack = Vec::new();
    for s in 0..n {
        if !flat[s] || seen[s] {
            continue;
        }
        seen[s] = true;
        stack.push(s);
        let mut size = 0usize;
        while let Some(i) = stack.pop() {
            size += 1;
            for j in [i - 1, i + 1, i - gw, i + gw] {
                if flat[j] && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        if size >= 16 {
            patches += 1;
            in_patches += size;
        }
    }
    (format!("eflat {:.4} patches {} patch_share {:.4}", nflat as f64 / land as f64, patches, in_patches as f64 / land as f64), flat)
}

/// [`flat_patches`]' flatness threshold in metres: far above the ε fill's own
/// step (a few millimetres at these heights) and far below any slope the
/// grid resolves (0.05 m over a 390 m cell is 0.007°).
const FLAT_EPS_M: f64 = 0.05;
/// [`flat_patches`]' lowest counted elevation, in metres above sea level.
const FLAT_MIN_M: f64 = 50.0;

/// A plain Lambertian hillshade of the final heights (light from the
/// north-west at 45 degrees, real metres, no exaggeration), grey, for looking
/// at the height field without the product's renderer. Sea is drawn flat
/// dark blue so the coast reads.
fn hillshade(ws: &cartalith_engine::WorldState, p: &cartalith_engine::WorldParams) -> Vec<u8> {
    let (gw, gh) = (p.gw, p.gh);
    let f = &ws.field;
    let sea = ws.sea_level;
    let k = p.peak_m / (1.0 - sea).max(1e-6);
    let cell_m = p.map_width_km * 1000.0 / gw as f64;
    let (lx, ly, lz) = (-0.5f64, -0.5f64, std::f64::consts::FRAC_1_SQRT_2);
    let mut rgb = vec![0u8; gw * gh * 3];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let o = i * 3;
            if (f[i] as f64) < sea {
                rgb[o..o + 3].copy_from_slice(&[30, 45, 80]);
                continue;
            }
            let at = |xx: usize, yy: usize| f[yy * gw + xx] as f64 * k;
            let (xl, xr) = (x.saturating_sub(1), (x + 1).min(gw - 1));
            let (yu, yd) = (y.saturating_sub(1), (y + 1).min(gh - 1));
            let dx = (at(xr, y) - at(xl, y)) / ((xr - xl).max(1) as f64 * cell_m);
            let dy = (at(x, yd) - at(x, yu)) / ((yd - yu).max(1) as f64 * cell_m);
            let n = (dx * dx + dy * dy + 1.0).sqrt();
            let s = ((-dx * lx - dy * ly + lz) / n).clamp(0.0, 1.0);
            let v = (s * 255.0).round() as u8;
            rgb[o..o + 3].copy_from_slice(&[v, v, v]);
        }
    }
    rgb
}

/// Each land cell's drainage terminal: follow `channels.recv` until a cell
/// with no receiver (or a sea cell) and record that cell's index. Sea cells,
/// and every cell on a world without channels, read `u32::MAX` ("not land").
/// A receiver chain longer than the grid is a cycle and panics.
fn terminals(ws: &cartalith_engine::WorldState) -> Option<Vec<u32>> {
    let c = ws.channels.as_ref()?;
    let f = &ws.field;
    let sea = ws.sea_level;
    let n = f.len();
    let mut out = vec![u32::MAX; n];
    let mut path = Vec::new();
    for start in 0..n {
        if (f[start] as f64) < sea || out[start] != u32::MAX {
            continue;
        }
        // Walk to a terminal or to a cell already resolved, then write the
        // answer back along the walked path (memoised, so the whole grid is
        // linear in its cell count).
        path.clear();
        let mut i = start;
        let t = loop {
            if out[i] != u32::MAX {
                break out[i];
            }
            path.push(i);
            assert!(path.len() <= n, "a receiver cycle");
            let r = c.recv[i];
            if r < 0 || (f[r as usize] as f64) < sea {
                break i as u32;
            }
            i = r as usize;
        };
        for &j in &path {
            out[j] = t;
        }
    }
    Some(out)
}

#[test]
#[ignore = "generates up to nine 2048x1311 worlds; run explicitly"]
fn mountain_relief_hstats() {
    let (gw, gh) = (
        std::env::var("MR_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(2048usize),
        std::env::var("MR_GH").ok().and_then(|v| v.parse().ok()).unwrap_or(1311usize),
    );
    let only = std::env::var("MR_WORLDS").ok();
    let out = std::env::var("MR_OUT").ok();
    let tag = std::env::var("MR_TAG").unwrap_or_default();
    let base = std::env::var("MR_BASE").ok();
    if let Some(o) = &out {
        std::fs::create_dir_all(o).unwrap();
    }
    for w in &WORLDS {
        if let Some(o) = &only
            && !o.split(',').any(|t| w.tag.starts_with(t))
        {
            continue;
        }
        let p = world_params(w, gw, gh);
        let t0 = std::time::Instant::now();
        let ws = cartalith_engine::generate_terrain(&p);
        let secs = t0.elapsed().as_secs_f64();
        let line = hstats(&ws, &p);
        let land: Vec<u8> = ws.field.iter().map(|&v| (v as f64 >= ws.sea_level) as u8).collect();
        let agree = match (&out, &base) {
            (Some(o), Some(b)) => match std::fs::read(format!("{o}/{}{b}_land.bin", w.tag)) {
                Ok(prev) if prev.len() == land.len() => {
                    let same = prev.iter().zip(&land).filter(|(a, b)| a == b).count();
                    format!(" | land_agree {:.5}", same as f64 / land.len() as f64)
                }
                _ => " | land_agree -".to_string(),
            },
            _ => String::new(),
        };
        // Watersheds: each land cell's terminal (the cell its receiver chain
        // ends at, `channels.recv`), compared against the base run's.
        let term = terminals(&ws);
        let bagree = match (&out, &base, &term) {
            (Some(o), Some(b), Some(t)) => match std::fs::read(format!("{o}/{}{b}_basin.bin", w.tag)) {
                Ok(prev) if prev.len() == t.len() * 4 => {
                    let prev: Vec<u32> = prev.chunks(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
                    let both: Vec<usize> = (0..t.len()).filter(|&i| t[i] != u32::MAX && prev[i] != u32::MAX).collect();
                    let same = both.iter().filter(|&&i| t[i] == prev[i]).count();
                    format!(" | basin_agree {:.5}", same as f64 / both.len().max(1) as f64)
                }
                _ => " | basin_agree -".to_string(),
            },
            _ => String::new(),
        };
        // FNV-1a over the final field's bits: identity of a switch-off arm
        // against an earlier build is this hash, not a statistic.
        let fnv = ws.field.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, v| v.to_bits().to_le_bytes().iter().fold(h, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)));
        println!("HSTAT {} {}{}{} | field {:016x} | gen {:.1}s", w.tag, line, agree, bagree, fnv, secs);
        if let Some(o) = &out {
            std::fs::write(format!("{o}/{}{tag}_land.bin", w.tag), &land).unwrap();
            if let Some(t) = &term {
                let bytes: Vec<u8> = t.iter().flat_map(|v| v.to_le_bytes()).collect();
                std::fs::write(format!("{o}/{}{tag}_basin.bin", w.tag), bytes).unwrap();
            }
            let mut rgb = hillshade(&ws, &p);
            // The flat cells (see `flat_patches`) drawn red over the hillshade.
            let sea = ws.sea_level;
            let (_, flat) = flat_patches(&ws.field, gw, gh, sea, if 1.0 - sea == 0.0 { 1e-6 } else { 1.0 - sea }, p.peak_m);
            for (i, &fl) in flat.iter().enumerate() {
                if fl {
                    rgb[i * 3..i * 3 + 3].copy_from_slice(&[255, 0, 0]);
                }
            }
            let png = cartalith_assets::raster::encode_png_rgb8(gw as u32, gh as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
            std::fs::write(format!("{o}/{}{tag}_hill.png", w.tag), png).unwrap();
        }
    }
}

/// The generation cost of `tect.orogen_relief` (Ruling CU item 1): the app's
/// `generate_terrain` at 2048 x 1311 with the switch off and on, alternated
/// so drift hits both arms, five runs each, on 71077345 classic and 314159
/// earth at 800 km. Prints median (min..max) per arm. Run **alone**, in
/// release (`MISTAKES.md`, "Quote a timing").
///
/// Protects: nothing (a measurement, `#[ignore]`d).
#[test]
#[ignore = "times ten 2048x1311 generations per world; run alone in release"]
fn mountain_relief_cost() {
    for w in WORLDS.iter().filter(|w| w.tag == "s71077345_classic_800" || w.tag == "s314159_earth_800") {
        let mut t = [Vec::new(), Vec::new()];
        for _ in 0..5 {
            for (k, on) in [false, true].into_iter().enumerate() {
                let mut p = world_params(w, 2048, 1311);
                p.tect.orogen_relief = on;
                let t0 = std::time::Instant::now();
                let ws = cartalith_engine::generate_terrain(&p);
                t[k].push(t0.elapsed().as_secs_f64());
                assert!(!ws.field.is_empty());
            }
        }
        for v in &mut t {
            v.sort_by(|a, b| a.total_cmp(b));
        }
        println!(
            "COST {}: off {:.2} s ({:.2}..{:.2}), on {:.2} s ({:.2}..{:.2})",
            w.tag, t[0][2], t[0][0], t[0][4], t[1][2], t[1][0], t[1][4]
        );
    }
}

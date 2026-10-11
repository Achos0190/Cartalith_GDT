//! **New World seed preview: acceptance harness** (`NEW_WORLD_PREVIEW_PROPOSAL.md`
//! NWP-1, Ruling CE). Promoted from the proposal's 2026-10-11 measurement harness: it now
//! measures the shipped `cartalith_engine::preview::land_mask` itself rather than a
//! hand-built copy of the early stages, so what it times and scores is what the dialog draws.
//!
//! It measures; it changes no generated output. Every test here is `#[ignore]`d. Run it
//! alone (no other `cargo`, `rustc` or Godot process), in release:
//!
//! ```text
//! cargo test --release -p cartalith-godot --test nwprev_cost -- --ignored --nocapture --test-threads=1
//! ```
//!
//! # What it answers
//!
//! 1. **Cost** (`nwprev_cost`, NWP-1 bar iv): `preview::land_mask` at 320 wide, median
//!    with min..max. Printed, not asserted: a timing assert under an unknown machine load
//!    proves nothing (`MISTAKES.md`, "Quote a timing").
//! 2. **Faithfulness** (`nwprev_overlap`, NWP-1 bars ii and iii): whether the 320-wide
//!    mask predicts the finished world (`generate_terrain` at the target width, same seed,
//!    same parameters). Agreement is printed beside the agreement two *unrelated* masks
//!    with the same land fractions would reach by chance, and beside a different-seed
//!    negative control, because a bare percentage of a mostly-ocean map is not evidence.
//!    **Asserts** the bars -- median >= 96 %, min >= 92 % per case -- after printing every
//!    row, so a miss is reported with its numbers rather than hidden.
//! 3. **Identity** (`nwprev_generate_identity`, NWP-1 acceptance i): a hash of every field
//!    of whole `generate_terrain` outputs, compared by hand across the substrate extract.
//!
//! # The arms of `nwprev_overlap`
//!
//! - `scaled` -- `preview::land_mask`: the substrate at 320 with both cell-unit corrections.
//! - `naive` -- the same function asked for a 320-wide world directly (its correction ratio
//!   is then 1, so neither correction applies): what the preview would be without them.
//! - `plates` -- the plate-sign mask (`build_plates` + `assign_plates`, the continental
//!   sign of each cell's plate): the proposal's §4.3 fallback, measured so it is ready.
//! - reference: the finished world at the target width (`NWPREV_TARGET`, default 1024),
//!   CPU path. `NWPREV_GPU=1` adds the GPU path, which the shipped shell builds: the
//!   preview with `use_gpu` on against the GPU-generated world (bar-checked too), plus two
//!   context rows -- the CPU preview against the GPU world, and the CPU world against the
//!   GPU world (`DECISIONS.md` §7c: different noise, a different world for the same seed).

#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use cartalith_engine::preview::{is_land, land_mask, preview_height};
use cartalith_engine::{generate_terrain, WorldParams, WorldStructureParams};
use cartalith_terrain::{assign_plates, build_plates, compute_warp, generate_continentality_field, WorldStructure};
use std::time::Instant;

/// The preview's width in cells (the owner's "320,px resolution").
const PREVIEW_W: usize = 320;
/// The default comparison width: the smallest resolution preset above 512 (`RESOLUTION_PRESETS`).
/// `NWPREV_TARGET` overrides it for `nwprev_overlap` (e.g. 2048, the dialog's default); an
/// unparsable value panics rather than falling back.
const TARGET_W: usize = 1024;

/// NWP-1 bar (ii): the median agreement every case must reach, in percent. A labelled
/// judgement set just under the proposal's measured 97.6-98.2 medians (§8).
const BAR_MEDIAN: f64 = 96.0;
/// NWP-1 bar (ii): the minimum agreement every case must reach, in percent. Same
/// judgement, just under the proposal's measured worst case of 93.0 (§8).
const BAR_MIN: f64 = 92.0;

/// The comparison width for this run: `NWPREV_TARGET` when set, else `TARGET_W`.
fn target_w() -> usize {
    match std::env::var("NWPREV_TARGET") {
        Ok(v) => v.parse::<usize>().unwrap_or_else(|_| panic!("NWPREV_TARGET: cannot parse `{v}`")),
        Err(_) => TARGET_W,
    }
}

/// Seeds measured. A labelled judgement: five arbitrary values, including the
/// seed the owner's own glass report quoted (51127).
const SEEDS: [i32; 5] = [1, 42, 1234, 51127, 987_654];

/// `reference_grid_h` in `cartalith-godot/src/lib.rs`: 2:1 for a world, 0.64 for a region.
fn grid_h(gw: usize, world: bool) -> usize {
    let k = if world { 0.5 } else { 0.64 };
    ((gw as f64 * k).round() as usize).max(4)
}

/// The app's own parameters (`params::defaults()`) for one preview case.
fn app_params(seed: i32, km: f64, world: bool, plates: usize, gw: usize) -> WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = grid_h(gw, world);
    p.world = world;
    p.tect.seed = seed;
    p.tect.plates = plates;
    p.map_width_km = km;
    p
}

/// `ARCHETYPES` in `cartalith-godot/src/lib.rs` (the reference's five presets), as
/// `(name, continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density)`.
/// Copied because that table is private to the cdylib; `the_binding_path_equals_the_engine_preview`
/// (lib.rs) drives the real table.
const ARCHETYPES: [(&str, [f64; 5]); 5] = [
    ("earth", [0.30, 0.50, 0.60, 0.60, 0.20]),
    ("supercontinent", [0.60, 0.10, 0.50, 0.70, 0.10]),
    ("archipelago", [0.15, 0.90, 0.80, 0.30, 0.50]),
    ("volcanic", [0.05, 1.00, 0.90, 0.80, 1.00]),
    ("rift", [0.40, 0.35, 0.75, 0.55, 0.30]),
];

fn ws_params(k: [f64; 5]) -> WorldStructureParams {
    WorldStructureParams {
        enabled: true,
        continentality: k[0],
        fragmentation: k[1],
        tectonic_energy: k[2],
        ocean_depth: k[3],
        hotspot_density: k[4],
    }
}

/// The plate-sign mask (1 = continental plate): the §4.3 fallback arm. Mirrors the
/// substrate's plate steps, including World Structure's derived plate count and crust
/// reclassification; measurement-only, never shipped.
fn plates_only(p: &WorldParams) -> Vec<u8> {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let ws = &p.world_structure;
    let n = if ws.enabled { ((4.0 + ws.fragmentation * 24.0).round() as usize).clamp(4, 40) } else { p.tect.plates };
    let cf = ws
        .enabled
        .then(|| generate_continentality_field(gw, gh, world, p.tect.seed, ws.continentality, ws.fragmentation));
    let wsa = cf.as_ref().map(|c| WorldStructure { ocean_depth: ws.ocean_depth, continental_field: c.as_slice() });
    let warp = compute_warp(gw, gh, p.tect.seed, p.tect.warp, world);
    let (wx, wy) = match &warp {
        Some((a, b)) => (Some(a.as_slice()), Some(b.as_slice())),
        None => (None, None),
    };
    let plates = build_plates(gw, gh, p.tect.seed as u32, n, p.tect.lloyd, world, wsa);
    let pid = assign_plates(gw, gh, world, &plates, wx, wy);
    let out: Vec<u8> = pid.iter().map(|&i| u8::from(plates[i as usize].base > 0.0)).collect();
    assert!(!out.is_empty(), "empty plate mask");
    out
}

/// Land (1) per `preview::is_land`, the classification the preview itself uses.
fn mask(field: &[f32], sea: f64) -> Vec<u8> {
    let m: Vec<u8> = field.iter().map(|&v| u8::from(is_land(v, sea))).collect();
    assert!(!m.is_empty(), "empty mask");
    m
}

/// Agreement between a `lw`-wide mask and a `hw`-wide one, sampled at each low cell's
/// centre. Returns (agreement %, chance agreement %, land % low, land % high, land IoU %).
/// Panics on an empty mask: a silently-empty comparison is the trap `MISTAKES.md` names.
fn compare(lo: &[u8], lw: usize, lh: usize, hi: &[u8], hw: usize, hh: usize) -> (f64, f64, f64, f64, f64) {
    assert!(!lo.is_empty() && !hi.is_empty(), "empty mask");
    assert_eq!(lo.len(), lw * lh);
    assert_eq!(hi.len(), hw * hh);
    let (mut agree, mut land_lo, mut land_hi, mut both, mut either) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for y in 0..lh {
        let hy = (((y as f64 + 0.5) * hh as f64 / lh as f64) as usize).min(hh - 1);
        for x in 0..lw {
            let hx = (((x as f64 + 0.5) * hw as f64 / lw as f64) as usize).min(hw - 1);
            let a = lo[y * lw + x];
            let b = hi[hy * hw + hx];
            agree += usize::from(a == b);
            land_lo += a as usize;
            land_hi += b as usize;
            both += usize::from(a == 1 && b == 1);
            either += usize::from(a == 1 || b == 1);
        }
    }
    let n = (lw * lh) as f64;
    let (pl, ph) = (land_lo as f64 / n, land_hi as f64 / n);
    let chance = pl * ph + (1.0 - pl) * (1.0 - ph);
    let iou = if either == 0 { f64::NAN } else { both as f64 / either as f64 };
    (100.0 * agree as f64 / n, 100.0 * chance, 100.0 * pl, 100.0 * ph, 100.0 * iou)
}

fn fmt_cmp(name: &str, c: (f64, f64, f64, f64, f64)) -> String {
    format!(
        "{name:<34} agree {:5.1}%  chance {:5.1}%  land {:4.1}% vs {:4.1}%  landIoU {:5.1}%",
        c.0, c.1, c.2, c.3, c.4
    )
}

/// Median, min and max of a set of samples, sorted in place.
fn med_min_max(v: &mut [f64]) -> (f64, f64, f64) {
    assert!(!v.is_empty(), "no samples");
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

/// Median, min and max of a set of millisecond samples, as text.
fn spread(mut v: Vec<f64>) -> String {
    let n = v.len();
    let (m, lo, hi) = med_min_max(&mut v);
    format!("median {m:.1} ms ({lo:.1}..{hi:.1}, n={n})")
}

/// One measured case: a label, map width (km), world flag, plate count and archetype.
type Case = (String, f64, bool, usize, Option<WorldStructureParams>);

/// The two cases the owner named (a 200 km region at 6 plates, an Earth-width world at 36),
/// then each of the five archetypes on the dialog's default 800 km region (NWP-1 bar ii).
fn cases() -> Vec<Case> {
    let mut v: Vec<Case> =
        vec![("region 200 km, 6 plates".into(), 200.0, false, 6, None), ("world 40 075 km, 36 plates".into(), 40075.0, true, 36, None)];
    for (name, k) in ARCHETYPES {
        v.push((format!("archetype {name}, region 800 km"), 800.0, false, 14, Some(ws_params(k))));
    }
    v
}

fn case_params(c: &Case, seed: i32, gw: usize) -> WorldParams {
    let mut p = app_params(seed, c.1, c.2, c.3, gw);
    if let Some(ws) = &c.4 {
        p.world_structure = ws.clone();
    }
    p
}

/// Cost of `preview::land_mask` at 320 wide for each case, against the dialog's default
/// 2048-wide target. Protects nothing; prints medians with min..max. Run ALONE.
#[test]
#[ignore = "measurement: run alone in release (see the module doc)"]
fn nwprev_cost() {
    // `NWPREV_GPU=1` times the GPU path (the shipped shell's default); the warm-up call
    // below then absorbs the one-off device handshake, which is reported separately.
    let gpu = std::env::var("NWPREV_GPU").is_ok_and(|v| v == "1");
    println!("rayon threads: {}  path: {}", rayon::current_num_threads(), if gpu { "GPU" } else { "CPU" });
    for c in cases() {
        let mut warm = case_params(&c, 51127, 2048);
        warm.use_gpu = gpu;
        let t = Instant::now();
        let _ = land_mask(&warm, PREVIEW_W); // warm-up (thread pool, page faults, GPU device)
        println!("WARMUP {}: {:.1} ms", c.0, t.elapsed().as_secs_f64() * 1000.0);
        let mut e = Vec::new();
        for s in SEEDS.iter().chain(SEEDS.iter()) {
            let mut p = case_params(&c, *s, 2048);
            p.use_gpu = gpu;
            let t = Instant::now();
            let m = land_mask(&p, PREVIEW_W);
            e.push(t.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(m.len(), PREVIEW_W * preview_height(&p, PREVIEW_W).unwrap());
        }
        println!("COST {}: land_mask {}x{} for a 2048x{} target: {}", c.0, PREVIEW_W, grid_h(PREVIEW_W, c.2), grid_h(2048, c.2), spread(e));
    }
}

/// FNV-1a, 64-bit: a fixed, dependency-free byte hash, so a hash printed by one build can be
/// compared with one printed by another (std's `DefaultHasher` makes no such promise).
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325) // FNV-1a 64 offset basis (the published constant)
    }
    fn bytes(&mut self, b: &[u8]) {
        // Length first, so two fields cannot trade bytes across their boundary.
        for &x in (b.len() as u64).to_le_bytes().iter().chain(b) {
            self.0 ^= x as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3); // FNV-1a 64 prime
        }
    }
    fn f32s(&mut self, v: &[f32]) {
        let b: Vec<u8> = v.iter().flat_map(|x| x.to_bits().to_le_bytes()).collect();
        self.bytes(&b);
    }
}

/// One hash over **every** field of a `WorldState`: each grid by its exact bits, the
/// optional grids with a presence tag, and the geology column by its `Debug` text (Rust's
/// `Debug` prints a float in its shortest round-trip form, so two different bit patterns
/// never print the same text, except NaN payloads, which no geology field holds). A field
/// left out here would be a field the extract could move unseen.
fn world_hash(ws: &cartalith_engine::WorldState) -> u64 {
    let mut h = Fnv::new();
    h.bytes(&ws.sea_level.to_bits().to_le_bytes());
    h.f32s(&ws.field);
    h.bytes(&ws.plate_id.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    h.bytes(&ws.boundary_mask);
    h.f32s(&ws.stress_field);
    h.f32s(&ws.age_field);
    h.f32s(&ws.resistance_field);
    h.f32s(&ws.crust_field);
    h.bytes(&ws.boundary_type);
    h.f32s(&ws.shear_field);
    h.f32s(&ws.volcanic_field);
    h.f32s(&ws.impact_field);
    h.f32s(&ws.temperature);
    h.f32s(&ws.rainfall);
    h.f32s(&ws.flow_discharge);
    h.bytes(&[u8::from(ws.integrated_drainage)]);
    match &ws.channels {
        Some(c) => {
            h.bytes(&[1]);
            h.bytes(&c.recv.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
            h.bytes(&c.chan);
            h.f32s(&c.slope);
            h.f32s(&c.intensity);
        }
        None => h.bytes(&[0]),
    }
    match &ws.stream_order {
        Some(v) => h.bytes(&v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>()),
        None => h.bytes(&[0xff]),
    }
    match &ws.river_mask {
        Some(v) => h.bytes(v),
        None => h.bytes(&[0xfe]),
    }
    match &ws.river_floor {
        Some(v) => h.f32s(v),
        None => h.bytes(&[0xfd]),
    }
    h.bytes(ws.gpu_stages_used.join(",").as_bytes());
    h.bytes(format!("{:?}", ws.geology).as_bytes());
    h.0
}

/// **NWP-1 acceptance (i)**: prints one hash of the whole `generate_terrain` output per case,
/// so the hashes printed before the substrate extract and after it can be compared line by
/// line (identity is proven by the two printouts agreeing, not by this test passing).
/// Cases: 3 seeds x 2 extents (an 800 km region, a 40 075 km world) x {Classic, the `earth`
/// archetype} on the app's parameters, plus the `WorldParams::defaults` parity baseline;
/// CPU, then GPU (`use_gpu = true`) -- each GPU row says which stages actually ran there, so
/// a machine without an adapter cannot pass a CPU fallback off as a GPU proof.
/// Protects: that the extract moved no byte of generated output. Run alone, in release.
#[test]
#[ignore = "measurement: run alone in release (see the module doc)"]
fn nwprev_generate_identity() {
    const W: usize = 512; // labelled judgement: large enough for every stage to act, small enough to run 50 worlds
    let earth = cartalith_engine::WorldStructureParams {
        enabled: true,
        continentality: 0.30,
        fragmentation: 0.50,
        tectonic_energy: 0.60,
        ocean_depth: 0.60,
        hotspot_density: 0.20,
    }; // `ARCHETYPES["earth"]`, `cartalith-godot/src/lib.rs`
    for gpu in [false, true] {
        for seed in [1, 42, 51127] {
            for (extent, km, world) in [("region 800 km", 800.0, false), ("world 40075 km", 40075.0, true)] {
                for arche in [false, true] {
                    let mut p = app_params(seed, km, world, 14, W);
                    p.use_gpu = gpu;
                    if arche {
                        p.world_structure = earth.clone();
                    }
                    let ws = generate_terrain(&p);
                    assert!(!ws.field.is_empty());
                    println!(
                        "IDENT gpu={gpu} seed={seed} {extent} archetype={} hash={:016x} gpu_stages=[{}]",
                        if arche { "earth" } else { "classic" },
                        world_hash(&ws),
                        ws.gpu_stages_used.join(",")
                    );
                }
            }
            let mut d = WorldParams::defaults(W, grid_h(W, false), seed);
            d.use_gpu = gpu;
            let ws = generate_terrain(&d);
            println!(
                "IDENT gpu={gpu} seed={seed} parity-defaults hash={:016x} gpu_stages=[{}]",
                world_hash(&ws),
                ws.gpu_stages_used.join(",")
            );
        }
    }
}

/// **NWP-1 bars (ii) and (iii).** Mask agreement between every preview arm and the finished
/// world at the target width, per seed and case, with a chance row and a different-seed
/// negative control on every line; then a per-case summary and the bar check.
/// Protects: that the shipped preview predicts the world Create builds (median >= 96 %,
/// min >= 92 %), and -- by mutation -- that each cell-unit correction is load-bearing.
/// Panics only after printing everything. Run ALONE.
#[test]
#[ignore = "measurement: run alone in release (see the module doc)"]
fn nwprev_overlap() {
    let target = target_w();
    let with_gpu = std::env::var("NWPREV_GPU").is_ok_and(|v| v == "1");
    let mut misses: Vec<String> = Vec::new();
    for c in cases() {
        let (lw, lh, hw, hh) = (PREVIEW_W, grid_h(PREVIEW_W, c.2), target, grid_h(target, c.2));
        println!("== {}  (preview {lw}x{lh}, target {hw}x{hh})", c.0);
        let (mut scaled_a, mut naive_a, mut plates_a, mut chance_a, mut neg_a, mut gpu_a) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        // Land IoU per arm: on a mostly-ocean map (the volcanic archetype is ~5 % land, chance
        // ~90 %) agreement is dominated by the ocean, and IoU is what shows the land itself.
        let (mut scaled_iou, mut plates_iou) = (Vec::new(), Vec::new());
        let mut prev_full: Option<(Vec<u8>, i32)> = None;
        for s in SEEDS {
            let ph = case_params(&c, s, target);
            let scaled = land_mask(&ph, PREVIEW_W);
            assert_eq!(scaled.len(), lw * lh, "preview size");
            let naive = land_mask(&case_params(&c, s, PREVIEW_W), PREVIEW_W);
            let plates = plates_only(&case_params(&c, s, PREVIEW_W));
            let full = generate_terrain(&ph);
            let m_full = mask(&full.field, full.sea_level);
            println!("-- seed {s} (finished sea level {:.4})", full.sea_level);
            let sc = compare(&scaled, lw, lh, &m_full, hw, hh);
            println!("{}", fmt_cmp("scaled (preview) vs finished", sc));
            let na = compare(&naive, lw, lh, &m_full, hw, hh);
            println!("{}", fmt_cmp("naive            vs finished", na));
            let pl = compare(&plates, lw, lh, &m_full, hw, hh);
            println!("{}", fmt_cmp("plates only      vs finished", pl));
            println!("CHANCE seed {s}: {:5.1}%", sc.1);
            scaled_a.push(sc.0);
            scaled_iou.push(sc.4);
            plates_iou.push(pl.4);
            naive_a.push(na.0);
            plates_a.push(pl.0);
            chance_a.push(sc.1);
            if with_gpu {
                let mut pg = ph.clone();
                pg.use_gpu = true;
                let g = generate_terrain(&pg);
                let m_gpu = mask(&g.field, g.sea_level);
                // The preview follows `use_gpu`, as Create does (`preview.rs` module doc).
                let gpu_preview = land_mask(&pg, PREVIEW_W);
                let gc = compare(&gpu_preview, lw, lh, &m_gpu, hw, hh);
                println!("{}  gpu_stages={}", fmt_cmp("GPU preview vs GPU finished", gc), g.gpu_stages_used.len());
                println!("{}", fmt_cmp("CPU preview vs GPU finished", compare(&scaled, lw, lh, &m_gpu, hw, hh)));
                println!("{}", fmt_cmp("CPU finished vs GPU finished", compare(&m_full, hw, hh, &m_gpu, hw, hh)));
                // Attribution: the GPU substrate at the TARGET width against the GPU world.
                // Low here means the stages after the substrate moved the coast (not the
                // preview's resolution); high here and low above means the resolution did.
                let gpu_at_target = land_mask(&pg, target);
                println!("{}", fmt_cmp("GPU substrate@target vs GPU finished", compare(&gpu_at_target, hw, hh, &m_gpu, hw, hh)));
                let cpu_at_target = land_mask(&ph, target);
                println!("{}", fmt_cmp("CPU substrate@target vs CPU finished", compare(&cpu_at_target, hw, hh, &m_full, hw, hh)));
                gpu_a.push(gc.0);
            }
            match &prev_full {
                Some((other, os)) => {
                    let ng = compare(&scaled, lw, lh, other, hw, hh);
                    println!("{}", fmt_cmp(&format!("NEG scaled vs finished seed {os}"), ng));
                    neg_a.push(ng.0);
                }
                None => println!("NEG seed {s}: no previous seed in this case (first row)"),
            }
            prev_full = Some((m_full, s));
        }
        let (sm, slo, shi) = med_min_max(&mut scaled_a);
        let (nm, nlo, nhi) = med_min_max(&mut naive_a);
        let (pm, plo, phi) = med_min_max(&mut plates_a);
        let (_, clo, chi) = med_min_max(&mut chance_a);
        let (_, glo, ghi) = med_min_max(&mut neg_a);
        let (si, silo, sihi) = med_min_max(&mut scaled_iou);
        let (pi, pilo, pihi) = med_min_max(&mut plates_iou);
        let verdict = if sm >= BAR_MEDIAN && slo >= BAR_MIN { "PASS" } else { "MISS" };
        println!(
            "SUMMARY target {target} | {} | scaled {sm:.1} ({slo:.1}..{shi:.1}) {verdict} | naive {nm:.1} ({nlo:.1}..{nhi:.1}) | plates {pm:.1} ({plo:.1}..{phi:.1}) | chance {clo:.1}..{chi:.1} | neg {glo:.1}..{ghi:.1} | landIoU scaled {si:.1} ({silo:.1}..{sihi:.1}) plates {pi:.1} ({pilo:.1}..{pihi:.1})",
            c.0
        );
        if with_gpu {
            let (gm, gl, gh) = med_min_max(&mut gpu_a);
            let gv = if gm >= BAR_MEDIAN && gl >= BAR_MIN { "PASS" } else { "MISS" };
            println!("SUMMARY-GPU target {target} | {} | GPU preview vs GPU world {gm:.1} ({gl:.1}..{gh:.1}) {gv}", c.0);
            if gv == "MISS" {
                misses.push(format!("{} (GPU path): median {gm:.1} (bar {BAR_MEDIAN}), min {gl:.1} (bar {BAR_MIN})", c.0));
            }
        }
        if verdict == "MISS" {
            misses.push(format!("{}: median {sm:.1} (bar {BAR_MEDIAN}), min {slo:.1} (bar {BAR_MIN})", c.0));
        }
    }
    assert!(misses.is_empty(), "NWP-1 bar (ii) missed at target {target}:\n{}", misses.join("\n"));
}

//! Deterministic A/B appearance dump — `TERRAIN_APPEARANCE_RESEARCH.md`
//! §1.6 ("establish deterministic A/B comparison rendering") and §32
//! (validate across contrasting terrain types, because appearance work that
//! flatters one often destroys another).
//!
//! Renders the *same* generated world twice — once through
//! `TerrainAppearance::js_reference()` (the reference HTML's single-sun, no-AO
//! shading, i.e. exactly what this renderer produced before milestone 2) and
//! once through `TerrainAppearance::default()` (the current look) — and
//! writes both to raw RGB files for side-by-side inspection.
//!
//! `#[ignore]` by default: it generates real worlds, which takes seconds and
//! has no business slowing `cargo test --workspace`. Run explicitly:
//!
//! ```text
//! cargo test -p cartalith-godot --test appearance_ab_dump -- --ignored --nocapture
//! ```
//!
//! Output: `target/ab/<world>_<before|after>.raw`, each `u32` width + `u32`
//! height (little-endian) followed by tightly-packed `RGB8` rows.

use rayon::prelude::*;
use std::fs;
use std::io::Write;

#[path = "../src/render.rs"]
mod render;

use cartalith_engine::{generate_terrain, WorldParams};

/// Grid size for the dump. Large enough that hillshade/AO read the way they
/// do in the real app, small enough to stay a few seconds per world.
/// Overridable with `CARTALITH_AB_N` so a milestone can measure at the app's
/// own 2048² (which is where milestone 4's published numbers were taken)
/// without editing this file.
fn grid_n() -> usize {
    std::env::var("CARTALITH_AB_N").ok().and_then(|v| v.parse().ok()).unwrap_or(512)
}

fn dump(name: &str, ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool, appearance: render::TerrainAppearance) {
    let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
    let ctx = render::RenderCtx::with_appearance(
        &ws.field,
        &ws.temperature,
        &ws.rainfall,
        Some(&ws.flow_discharge),
        gw,
        gh,
        ws.sea_level,
        world,
        55.0,
        5.0,
        appearance.clone(),
    )
    .with_lithology(&lith);
    let mut header = Vec::with_capacity(8);
    header.extend_from_slice(&(gw as u32).to_le_bytes());
    header.extend_from_slice(&(gh as u32).to_le_bytes());
    // Milestone 6: row-parallel, mirroring `lib.rs`'s own loop, so the
    // printed `render_ms` is what the real app now pays. The pre-milestone-6
    // serial cost of the same image is printed separately by
    // `time_serial_vs_parallel`, which also asserts the two are byte-equal.
    let mut rgb = vec![0u8; gw * gh * 3];
    let t0 = std::time::Instant::now();
    rgb.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            let o = x * 3;
            row[o] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
            row[o + 1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
            row[o + 2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    });
    // Milestone 5 (§18): the one stage that runs over the finished raster
    // rather than per pixel. Mirrors `lib.rs`'s own call so the dump and the
    // real app show the same image.
    render::apply_local_contrast(&appearance, &mut rgb, gw, gh, world);
    let render_ms = t0.elapsed().as_secs_f64() * 1000.0;
    fs::create_dir_all("../../target/ab").unwrap();
    let path = format!("../../target/ab/{name}.raw");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(&header).unwrap();
    f.write_all(&rgb).unwrap();
    println!("wrote {path} ({gw}x{gh}) render {render_ms:.0} ms");
}

/// The one cross-crate check `render.rs` itself cannot make: it is
/// `#[path]`-included standalone by `golden_parity_render.rs`, so it spells
/// the rock-type order out as its own const rather than importing
/// `cartalith_civ::LITH_KEYS`. This test target *can* see both, so the
/// duplicate is verified rather than assumed — if `build_lithology`'s
/// vocabulary is ever reordered or extended, this fails instead of the map
/// silently painting basalt where limestone is.
///
/// Not `#[ignore]`d: it generates nothing and runs in microseconds, so it
/// belongs in the ordinary `cargo test --workspace` sweep.
#[test]
fn litho_palette_order_matches_civ_vocabulary() {
    assert_eq!(render::LITHO_PALETTE_ORDER.len(), cartalith_civ::LITH_KEYS.len(), "lithology vocabulary grew; render.rs needs a palette for every type");
    for (i, (mine, theirs)) in render::LITHO_PALETTE_ORDER.iter().zip(cartalith_civ::LITH_KEYS.iter()).enumerate() {
        assert_eq!(mine, theirs, "lithology index {i} disagrees between render.rs and cartalith-civ");
    }
}

/// Milestone 6 (research §21/§23): the same world, the same appearance,
/// rendered once through the pre-milestone-6 serial loop and once through the
/// row-parallel one. Prints both and **asserts they are byte-equal**, which is
/// the determinism claim (§27) measured rather than asserted in prose.
fn time_serial_vs_parallel(label: &str, ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool) {
    let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
    let a = render::TerrainAppearance::default();
    let ctx = render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, world, 55.0, 5.0, a.clone()).with_lithology(&lith);

    let mut ser = vec![0u8; gw * gh * 3];
    let t0 = std::time::Instant::now();
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            let o = (y * gw + x) * 3;
            ser[o] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
            ser[o + 1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
            ser[o + 2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    let serial_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let mut par = vec![0u8; gw * gh * 3];
    let t1 = std::time::Instant::now();
    par.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            let o = x * 3;
            row[o] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
            row[o + 1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
            row[o + 2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    });
    let parallel_ms = t1.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(ser, par, "{label}: parallel render is not byte-identical to serial");

    // And the whole-raster stage, which parallelizes separately.
    let mut lc = par.clone();
    let t2 = std::time::Instant::now();
    render::apply_local_contrast(&a, &mut lc, gw, gh, world);
    let lc_ms = t2.elapsed().as_secs_f64() * 1000.0;

    println!("{label} {gw}x{gh} cell_color serial {serial_ms:.0} ms -> parallel {parallel_ms:.0} ms ({:.1}x), local_contrast {lc_ms:.0} ms", serial_ms / parallel_ms.max(1e-9));
}

/// Marginal cost of each appearance stage: the full `default()` look rendered
/// with exactly **one** stage disabled, timed against the full render. This is
/// what the §29 tier ladder is designed from -- milestone 6 found by measuring
/// it that the research doc's own intuition about which stages are expensive
/// is wrong for this renderer (multidirectional lighting and AO are nearly
/// free; the whole-raster local-contrast pass is the single largest item), and
/// a tier table built on the intuition instead of the measurement would have
/// given up the legibility for none of the cost.
fn cost_table(label: &str, ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool) {
    let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
    let d = render::TerrainAppearance::default();

    // Best of three. A single wall-clock sample at this size is dominated by
    // scheduler and thermal noise -- the first version of this table produced
    // *negative* marginal costs, which is the measurement telling you it is
    // not a measurement. The minimum is the least-contaminated sample.
    let time_once = |a: &render::TerrainAppearance| -> f64 {
        let ctx = render::RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, world, 55.0, 5.0, a.clone()).with_lithology(&lith);
        let mut rgb = vec![0u8; gw * gh * 3];
        let t0 = std::time::Instant::now();
        rgb.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
            for x in 0..gw {
                let (r, g, b) = render::cell_color(&ctx, x, y);
                let o = x * 3;
                row[o] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
                row[o + 1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
                row[o + 2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        });
        render::apply_local_contrast(a, &mut rgb, gw, gh, world);
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let time_one = |a: &render::TerrainAppearance| -> f64 { (0..3).map(|_| time_once(a)).fold(f64::INFINITY, f64::min) };

    let base = time_one(&d);
    let variants: Vec<(&str, render::TerrainAppearance)> = vec![
        ("relief_lights 6->1", render::TerrainAppearance { relief_lights: 1, ..d.clone() }),
        ("ao", render::TerrainAppearance { ao_strength: 0.0, ..d.clone() }),
        ("hydro tint", render::TerrainAppearance { hydro_wet_strength: 0.0, ..d.clone() }),
        ("paper grain", render::TerrainAppearance { paper_grain: 0.0, ..d.clone() }),
        ("paper mottle", render::TerrainAppearance { paper_mottle: 0.0, ..d.clone() }),
        ("stipple", render::TerrainAppearance { stipple_strength: 0.0, ..d.clone() }),
        ("geology", render::TerrainAppearance { litho_strength: 0.0, litho_exposure: 0.0, ..d.clone() }),
        ("local contrast", render::TerrainAppearance { local_contrast: 0.0, ..d.clone() }),
    ];
    println!("{label} {gw}x{gh} stage cost (full render {base:.0} ms):");
    for (name, v) in variants {
        let t = time_one(&v);
        println!("    {name:22} {:6.0} ms  ({:.0} ms without)", base - t, t);
    }
}

fn run(label: &str, mut p: WorldParams) {
    let gw = p.gw;
    let gh = p.gh;
    let world = p.world;
    p.use_gpu = false;
    let ws = generate_terrain(&p);
    dump(&format!("{label}_before"), &ws, gw, gh, world, render::TerrainAppearance::js_reference());
    dump(&format!("{label}_after"), &ws, gw, gh, world, render::TerrainAppearance::default());

    // Milestone 3 isolation pair: milestone 2's relief lighting/AO held
    // fixed at their own `default()` values, hydrology tint alone toggled
    // off vs on — isolates this milestone's own delta from milestone 2's
    // already-verified one, rather than conflating both against JS.
    let no_hydro = render::TerrainAppearance { hydro_wet_strength: 0.0, ..Default::default() };
    dump(&format!("{label}_nohydro"), &ws, gw, gh, world, no_hydro);
    dump(&format!("{label}_withhydro"), &ws, gw, gh, world, render::TerrainAppearance::default());

    // Milestone 4 isolation pair: milestones 2 and 3 held fixed at their
    // own `default()` values, the three atlas stages (paper ground, forest
    // stippling, plate border) toggled off vs on together — so this
    // milestone's delta is measured against the already-verified
    // milestone-3 look, not conflated with it.
    let no_atlas = render::TerrainAppearance { paper_strength: 0.0, stipple_strength: 0.0, border_width_frac: 0.0, ..Default::default() };
    dump(&format!("{label}_noatlas"), &ws, gw, gh, world, no_atlas);
    dump(&format!("{label}_withatlas"), &ws, gw, gh, world, render::TerrainAppearance::default());

    // And each atlas stage alone, since the three are independent and a
    // combined dump can't tell which one is carrying the change.
    let paper_only = render::TerrainAppearance { stipple_strength: 0.0, border_width_frac: 0.0, ..Default::default() };
    dump(&format!("{label}_paperonly"), &ws, gw, gh, world, paper_only);
    let stipple_only = render::TerrainAppearance { paper_strength: 0.0, border_width_frac: 0.0, ..Default::default() };
    dump(&format!("{label}_stippleonly"), &ws, gw, gh, world, stipple_only);

    // Milestone 5 isolation pair: milestones 2-4 held fixed at their own
    // `default()` values, geology (§12) and local contrast (§18) toggled off
    // vs on together — so this milestone's delta is measured against the
    // already-verified milestone-4 look, not conflated with it.
    let no_m5 = render::TerrainAppearance { litho_strength: 0.0, litho_exposure: 0.0, local_contrast: 0.0, ..Default::default() };
    dump(&format!("{label}_nom5"), &ws, gw, gh, world, no_m5);
    dump(&format!("{label}_withm5"), &ws, gw, gh, world, render::TerrainAppearance::default());

    // And each of the two alone, since they are independent stages acting on
    // different things (material identity vs. tonal separation) and a
    // combined image cannot say which one is carrying a change.
    let geo_only = render::TerrainAppearance { local_contrast: 0.0, ..Default::default() };
    dump(&format!("{label}_geoonly"), &ws, gw, gh, world, geo_only);
    let lc_only = render::TerrainAppearance { litho_strength: 0.0, litho_exposure: 0.0, ..Default::default() };
    dump(&format!("{label}_lconly"), &ws, gw, gh, world, lc_only);

    // Milestone 6 (§29): the four quality tiers of the same world, so the
    // ladder can be judged as four *looks* rather than as a cost table.
    // `tier_quality` is `after`/`withm5` again by construction, and dumping
    // it anyway is the cheapest possible proof of that.
    for tier in render::QualityTier::ALL {
        dump(&format!("{label}_tier_{}", tier.name()), &ws, gw, gh, world, render::TerrainAppearance::for_tier(tier));
    }

    time_serial_vs_parallel(label, &ws, gw, gh, world);
    cost_table(label, &ws, gw, gh, world);
}

fn archipelago(gw: usize, gh: usize) -> WorldParams {
    // The app's own archetype knobs for that preset (`cartalith-godot`'s
    // `generate_world_structure`, "archipelago" arm) — a genuinely
    // different, low-relief, fragmented world, which is the contrast §32
    // asks for.
    let mut p = WorldParams::defaults(gw, gh, 12345);
    p.world_structure.enabled = true;
    p.world_structure.continentality = 0.15;
    p.world_structure.fragmentation = 0.90;
    p.world_structure.tectonic_energy = 0.80;
    p.world_structure.ocean_depth = 0.30;
    p.world_structure.hotspot_density = 0.50;
    p
}

#[test]
#[ignore = "generates real worlds; run explicitly with --ignored"]
fn dump_ab_classic_and_archipelago() {
    let n = grid_n();
    // Classic: the plain, non-archetype pipeline — mountainous, one large
    // landmass at this seed (matches the app's own default view).
    run("classic", WorldParams::defaults(n, n, 12345));
    run("archipelago", archipelago(n, n));

    // Non-square (commit `22ae75b` made these real): a 2:1 landscape plate.
    // Every radius in `render.rs` is keyed to `gw`, so a wide short grid is
    // where a width-derived blur radius can exceed the map's own height —
    // the same class of bug the plate frame already had fixed here.
    run("wide", WorldParams::defaults(n, n / 2, 12345));
}

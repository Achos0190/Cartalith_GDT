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

use std::fs;
use std::io::Write;

#[path = "../src/render.rs"]
mod render;

use cartalith_engine::{generate_terrain, WorldParams};

/// Grid size for the dump. Large enough that hillshade/AO read the way they
/// do in the real app, small enough to stay a few seconds per world.
const N: usize = 512;

fn dump(name: &str, ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool, appearance: render::TerrainAppearance) {
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
        appearance,
    );
    let mut buf = Vec::with_capacity(8 + gw * gh * 3);
    buf.extend_from_slice(&(gw as u32).to_le_bytes());
    buf.extend_from_slice(&(gh as u32).to_le_bytes());
    let t0 = std::time::Instant::now();
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            buf.push((r * 255.0).round().clamp(0.0, 255.0) as u8);
            buf.push((g * 255.0).round().clamp(0.0, 255.0) as u8);
            buf.push((b * 255.0).round().clamp(0.0, 255.0) as u8);
        }
    }
    let render_ms = t0.elapsed().as_secs_f64() * 1000.0;
    fs::create_dir_all("../../target/ab").unwrap();
    let path = format!("../../target/ab/{name}.raw");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(&buf).unwrap();
    println!("wrote {path} ({gw}x{gh}) render {render_ms:.0} ms");
}

fn run(label: &str, mut p: WorldParams) {
    let gw = p.gw;
    let gh = p.gh;
    let world = p.world;
    p.use_gpu = false;
    let ws = generate_terrain(&p);
    dump(&format!("{label}_before"), &ws, gw, gh, world, render::TerrainAppearance::js_reference());
    dump(&format!("{label}_after"), &ws, gw, gh, world, render::TerrainAppearance::default());
}

#[test]
#[ignore = "generates real worlds; run explicitly with --ignored"]
fn dump_ab_classic_and_archipelago() {
    // Classic: the plain, non-archetype pipeline — mountainous, one large
    // landmass at this seed (matches the app's own default view).
    run("classic", WorldParams::defaults(N, N, 12345));

    // Archipelago: the app's own archetype knobs for that preset
    // (`cartalith-godot`'s `generate_world_structure`, "archipelago" arm) —
    // a genuinely different, low-relief, fragmented world, which is the
    // contrast §32 asks for.
    let mut arch = WorldParams::defaults(N, N, 12345);
    arch.world_structure.enabled = true;
    arch.world_structure.continentality = 0.15;
    arch.world_structure.fragmentation = 0.90;
    arch.world_structure.tectonic_energy = 0.80;
    arch.world_structure.ocean_depth = 0.30;
    arch.world_structure.hotspot_density = 0.50;
    run("archipelago", arch);
}

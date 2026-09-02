//! The bake, end to end, over a **real generated world** rather than a
//! synthetic fixture — `GUI_GAP_REGISTER.md` WW-01's own acceptance:
//! bake a world, confirm the atlas persists, confirm a deep-zoom read comes
//! back from it, confirm the archive round-trips.
//!
//! `#[ignore]` because it runs a full `generate_terrain` and a depth-3 bake
//! (85 tiles, each rendered to PNG), which is minutes rather than
//! milliseconds. Run it deliberately:
//!
//! ```text
//! cargo test -p cartalith-engine --test bake_real_world -- --ignored --nocapture
//! ```
//!
//! Everything it asserts is also covered by the fast unit tests in
//! `bake.rs`; what this adds is that the inputs are a real world's height
//! field with real coastlines, mountains and river-carved valleys, and that
//! the numbers it prints are the ones the UI will show a user.

use cartalith_engine::bake::{
    atlas_export_entries, atlas_import_entries, bake_all_tiles, pyramid_tile, BakeOpts,
};
use cartalith_engine::{generate_terrain, WorldParams};
use cartalith_io::atlas::AtlasStore;
use cartalith_spatial::pyramid::ChunkId;
use cartalith_terrain::amplify::AmplifyOpts;

const SEED: i32 = 20260824;

/// Overridable from the environment so the same test can answer "what does
/// this cost on a *shipping-size* world" without a second copy of it:
///
/// ```text
/// CARTALITH_BAKE_GW=2048 CARTALITH_BAKE_GH=1311 CARTALITH_BAKE_TILE=1024 \
/// ```
///
/// The defaults are small enough to finish in under a second, which is what
/// makes this worth keeping runnable rather than a one-off script.
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("cartalith-bake-real-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
#[ignore = "generates a real world and bakes 85 tiles; run explicitly"]
fn a_real_world_bakes_persists_and_round_trips() {
    let gw = env_usize("CARTALITH_BAKE_GW", 384);
    let gh = env_usize("CARTALITH_BAKE_GH", 256);
    let tile = env_usize("CARTALITH_BAKE_TILE", 256);
    let depth = env_usize("CARTALITH_BAKE_DEPTH", 3) as i32;
    let expect = cartalith_spatial::pyramid::pyramid_tile_count(depth) as usize;
    let mut p = WorldParams::defaults(gw, gh, SEED);
    p.map_width_km = 800.0;
    let t_gen = std::time::Instant::now();
    let ws = generate_terrain(&p);
    println!("generate_terrain {gw}x{gh}: {:.2}s", t_gen.elapsed().as_secs_f64());

    // The world must actually have relief, or every assertion below passes
    // over a flat plate and means nothing.
    let lo = ws.field.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = ws.field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let land = ws.field.iter().filter(|&&v| v as f64 > ws.sea_level).count();
    println!("field range [{lo:.4}, {hi:.4}], {land} land cells of {}", gw * gh);
    assert!(hi - lo > 0.2, "the generated world is nearly flat: [{lo}, {hi}]");
    assert!(land > gw * gh / 20, "almost no land: {land} cells");

    let root = tmp("main");
    let store = AtlasStore::new(&root);
    let amplify = AmplifyOpts { seed: SEED, sea: ws.sea_level, ..Default::default() };
    let o = BakeOpts {
        world_key: "realworld",
        tile_size: tile,
        amplify: &amplify,
        visual: Some(cartalith_engine::region_export::TileVisual {
            sea: ws.sea_level,
            ..Default::default()
        }),
        version: "test",
    };

    let t0 = std::time::Instant::now();
    let mut progress_calls = 0u64;
    let report = bake_all_tiles(&ws.field, gw, gh, depth, &store, &o, |_, _| progress_calls += 1);
    let secs = t0.elapsed().as_secs_f64();
    println!(
        "bake depth {depth} @ {tile}px: {} baked, {} skipped, {} failed in {secs:.2}s ({:.0} ms/tile)",
        report.baked, report.skipped, report.failed, secs * 1000.0 / report.total().max(1) as f64
    );
    assert_eq!(report.baked, expect, "the pyramid's own (4^(depth+1)-1)/3");
    assert_eq!(report.failed, 0);
    assert_eq!(progress_calls, expect as u64, "one progress call per tile");

    // It persisted: a *fresh* store over the same directory sees all of it.
    let reopened = AtlasStore::new(&root);
    let keys = reopened.keys_for_world("realworld").unwrap();
    assert_eq!(keys.len(), expect, "the atlas did not survive being reopened");
    let bytes = reopened.world_bytes("realworld").unwrap();
    println!("atlas on disk: {} chunks, {:.2} MiB", keys.len(), bytes as f64 / 1048576.0);
    assert!(bytes as usize > expect * 1024, "an {expect}-chunk atlas cannot be {bytes} bytes");

    // A deep-zoom read really comes back from the store, and really is the
    // tile live synthesis would have produced.
    let id = ChunkId::new(depth as u32, 5.min((1u32 << depth) - 1), 3.min((1u32 << depth) - 1));
    let live = pyramid_tile(&ws.field, gw, gh, id, tile, &amplify);
    let stored = reopened.get("realworld", tile, id).unwrap().expect("baked");
    let back = cartalith_io::unpack_height16(&stored.rg16, live.w * live.h);
    let maxd = live.data.iter().zip(&back).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
    println!("deep-zoom read {id:?}: {}x{} px, max delta {maxd:.2e}", live.w, live.h);
    assert!(maxd <= 1.0 / 65535.0, "the stored tile is not the synthesised one ({maxd})");
    assert!(back.iter().any(|&v| v != back[0]), "the stored tile is flat");

    // The visual is a real, decodable PNG at the tile's own size.
    let png = stored.png.expect("a visual bake stores a PNG");
    let img = cartalith_assets::raster::decode_png(&png).expect("a valid PNG");
    assert_eq!((img.w as usize, img.h as usize), (live.w, live.h));
    println!("chunk visual: {}x{} PNG, {} bytes", img.w, img.h, png.len());

    // Re-baking the same depth writes nothing and skips everything.
    let again = bake_all_tiles(&ws.field, gw, gh, depth, &store, &o, |_, _| {});
    assert_eq!((again.baked, again.skipped), (0, expect), "a re-bake must be a no-op");

    // The portable archive round-trips into a fresh store, byte for byte.
    let t1 = std::time::Instant::now();
    let (entries, man) =
        atlas_export_entries(&store, "realworld", gw, gh, "test", 0, None, true).expect("export");
    let total: usize = entries.iter().map(|e| e.data.len()).sum();
    println!(
        "archive: {} entries, {:.2} MiB gzipped, in {:.2}s",
        entries.len(), total as f64 / 1048576.0, t1.elapsed().as_secs_f64()
    );
    assert_eq!(man.count, expect);
    assert_eq!(entries.len(), expect * 2 + 1, "one bin + one png per chunk, plus the manifest");

    let root2 = tmp("import");
    let store2 = AtlasStore::new(&root2);
    let lookup = |n: &str| entries.iter().find(|e| e.name == n).map(|e| e.data.clone());
    let (n, wk) = atlas_import_entries(&store2, &lookup).expect("a valid atlas");
    assert_eq!((n, wk.as_str()), (expect, "realworld"));
    for k in reopened.keys_for_world("realworld").unwrap() {
        let a = reopened.get("realworld", k.ts, k.id).unwrap().unwrap();
        let b = store2.get("realworld", k.ts, k.id).unwrap().unwrap();
        assert_eq!(a.rg16, b.rg16, "chunk {:?} height", k.id);
        assert_eq!(a.png, b.png, "chunk {:?} visual", k.id);
    }

    // Clearing frees it all.
    assert_eq!(store.clear_world("realworld").unwrap(), expect);
    assert!(store.keys_for_world("realworld").unwrap().is_empty());

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&root2);
}

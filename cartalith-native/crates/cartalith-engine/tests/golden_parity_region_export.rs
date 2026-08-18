//! Golden-parity test for `UNIFIED_TOOL_PLAN.md` milestone E's
//! `exportRegionTiles` composition (reference line 11891).
//!
//! # Disclosed: this is a composition, not a slice
//!
//! `exportRegionTiles` itself is `async` and calls `tilePngBytes`
//! (`OffscreenCanvas`) and `gzipBytes` (`CompressionStream`) inside its loop,
//! neither of which exists in the harness — so it is **not** invoked directly.
//! The harness instead runs the reference's own `tileDims`, `refineTile`,
//! `packHeight16` and `buildTileManifest`, in the reference's own order, with
//! the reference's own name templates, and records the resulting entry list.
//! That is weaker evidence than calling the function would be (milestone C
//! made the same disclosure about `sculptCommit`'s transcribed body), and it
//! is stated here rather than implied. What it *does* establish is exact: the
//! four primitives are bit-identical to the reference, and the port assembles
//! them into the same names, the same byte counts and the same manifest.
//!
//! Harness details: see `cartalith-terrain/tests/golden_parity_amplify.rs` —
//! whole `<script>` blocks, delimiters asserted, balance check fixed twice
//! rather than deleted, synthetic pure-arithmetic fixture field hashed on both
//! sides first.

use cartalith_engine::region_export::export_region_tiles;
use cartalith_spatial::Region;
use cartalith_terrain::amplify::AmplifyOpts;

fn fnv_u8(a: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in a {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
    let mut f = vec![0.0f32; gw * gh];
    let cx = gw as f64 * 0.42;
    let cy = gh as f64 * 0.55;
    let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
    for y in 0..gh {
        for x in 0..gw {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
            let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
            v += 0.05 * ((q as f64 / 10.0) - 0.5);
            v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
            f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
        }
    }
    f
}

#[test]
fn a_two_by_two_refine_matches_the_reference_entry_for_entry() {
    let src = synthetic_field(48, 32, 5);
    let e = export_region_tiles(
        &src,
        48,
        32,
        &Region { x: 4, y: 4, w: 24, h: 16 },
        2,
        2,
        32,
        &AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() },
        false,
        "TESTVER",
    );
    // tileDims picked these: a 12x8 coarse tile is wider than tall, so the
    // long edge takes ts=32 and the short one scales to 21.
    assert_eq!((e.tile_w, e.tile_h), (32, 21));

    let want = [
        ("tiles/refined_0_0_rg16.bin", "942adf3ae1952d6e", 2688usize),
        ("tiles/refined_0_1_rg16.bin", "4192b322e8668c86", 2688),
        ("tiles/refined_1_0_rg16.bin", "562d4e66f2e58118", 2688),
        ("tiles/refined_1_1_rg16.bin", "f9d4b01a7b453529", 2688),
    ];
    for (i, (name, hash, len)) in want.into_iter().enumerate() {
        assert_eq!(e.entries[i].name, name);
        assert_eq!(e.entries[i].data.len(), len);
        assert_eq!(fnv_u8(&e.entries[i].data), hash, "{name}");
    }

    assert_eq!(e.entries[4].name, "tiles/index.json");
    assert_eq!(fnv_u8(&e.entries[4].data), "a2c757b803ca92ad");
    assert_eq!(e.entries[4].data.len(), 1024);
}

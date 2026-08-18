//! Golden-parity test for `UNIFIED_TOOL_PLAN.md` milestones E and E2's
//! `exportRegionTiles` composition (reference line 11891).
//!
//! # Milestone E's disclosure, and how E2 discharged it
//!
//! Milestone E recorded, honestly, that it never invoked `exportRegionTiles`
//! itself: the function is `async` and calls `tilePngBytes`
//! (`OffscreenCanvas`) and `gzipBytes` (`CompressionStream`) inside its loop.
//! Its harness ran `tileDims`, `refineTile`, `packHeight16` and
//! `buildTileManifest` in the reference's own order instead, which established
//! that the four primitives are bit-identical and that the port assembles them
//! into the same names — but not that the reference's own loop does.
//!
//! **Milestone E2 ran the real function.** Node has `CompressionStream`, so
//! `gzipBytes` works unmodified; `tilePngBytes` finds no `OffscreenCanvas` and
//! answers `null`, which is precisely the headless behaviour the reference
//! documents ("*null headless/on failure (the export simply omits the PNG
//! layer)*"). So `exportRegionTiles(sel, 2, 2, 32, true, null)` was called for
//! real, and its four gzipped tiles gunzip to **exactly** milestone E's four
//! recorded hashes, on the same fixture field. The disclosure is discharged:
//! the assembly matches, not just its parts.
//!
//! # One harness bug found on the way, and it looked like a reference bug
//!
//! The first real call disagreed with milestone E on the *fourth* tile only.
//! Cause: with the DOM stubbed, block #1's boot code schedules a deferred
//! first `generate()` on a timer, and the reference's `microtask()` — which
//! `exportRegionTiles` awaits between tiles — is literally `setTimeout(r, 0)`.
//! The boot work therefore fired between tile 3 and tile 4 and overwrote
//! `field` mid-loop. `amplifyRegion` is not non-deterministic; the harness
//! was. Fixed by making `requestAnimationFrame` inert and draining pending
//! macrotasks before installing any fixture, after which all four tiles match.
//!
//! Harness details otherwise: see
//! `cartalith-terrain/tests/golden_parity_amplify.rs` — whole `<script>`
//! blocks, delimiters asserted, balance check fixed twice rather than deleted,
//! synthetic pure-arithmetic fixture field hashed on both sides first.

use cartalith_engine::region_export::{RegionExportOpts, export_region_tiles};
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

/// The reference world the harness set up: `GW=48`, `GH=32`, seed 4242, sea
/// level 0.42, ridged off, `sel = {x:4, y:4, w:24, h:16}`, a 2x2 grid at
/// `ts=32`.
fn run(gzip: bool, version: &str) -> cartalith_engine::region_export::RegionExport {
    let src = synthetic_field(48, 32, 5);
    let a = AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() };
    export_region_tiles(
        &src,
        48,
        32,
        &Region { x: 4, y: 4, w: 24, h: 16 },
        &RegionExportOpts {
            cols: 2,
            rows: 2,
            tile_size: 32,
            amplify: &a,
            world: false,
            version,
            gzip,
            // The reference's own headless behaviour: no canvas, no PNG layer.
            visual: None,
        },
    )
}

#[test]
fn a_two_by_two_refine_matches_the_reference_entry_for_entry() {
    let e = run(false, "TESTVER");
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

#[test]
fn the_gzip_path_matches_the_reference_name_for_name() {
    // Straight off a real `exportRegionTiles(sel, 2, 2, 32, true, null)` call.
    // The gzip BYTES are not compared -- two conforming deflate encoders need
    // not agree on a bit stream, and the reference stores the tile
    // uncompressed on any platform that has no CompressionStream at all. What
    // is compared is what a consumer actually depends on: the names, the
    // manifest's own record of the compression, and the payload after gunzip.
    let e = run(true, "2.10");
    assert_eq!(
        e.entries.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        vec![
            "tiles/refined_0_0_rg16.bin.gz",
            "tiles/refined_0_1_rg16.bin.gz",
            "tiles/refined_1_0_rg16.bin.gz",
            "tiles/refined_1_1_rg16.bin.gz",
            "tiles/index.json",
        ]
    );
    assert!(e.used_gzip);

    // The four hashes the REAL reference call produced after gunzip -- and the
    // same four milestone E recorded from its primitives-only harness.
    let want = [
        "942adf3ae1952d6e",
        "4192b322e8668c86",
        "562d4e66f2e58118",
        "f9d4b01a7b453529",
    ];
    for (i, hash) in want.into_iter().enumerate() {
        let plain = cartalith_io::gunzip_bytes(&e.entries[i].data).expect("valid gzip");
        assert_eq!(plain.len(), 2688, "tile {i} gunzips to a full 32x21 RG16 tile");
        assert_eq!(fnv_u8(&plain), hash, "tile {i}");
    }

    // The reference's index.json for this run, with `version` set to its own
    // VERSION so the byte count is directly comparable: 1020 bytes, and
    // `compression` reading "gzip".
    let json = String::from_utf8(e.entries[4].data.clone()).expect("utf-8");
    assert_eq!(json.len(), 1020, "manifest byte count");
    assert!(json.contains("\"compression\": \"gzip\""));
    assert!(json.contains("\"version\": \"2.10\""));
    // ...and the per-tile `file` still names the PNG, gzip or not.
    assert!(json.contains("\"file\": \"tiles/refined_1_1.png\""));
}

#[test]
fn extract_region_as_world_matches_the_reference_plan() {
    // `regionNewWorldBtn`'s two arithmetic lines (reference 13219), evaluated
    // against the reference's own `tileDims` for five worlds. The dimensions
    // and the km scale are the whole of what the handler computes before it
    // starts mutating state; the mutation itself is shell orchestration and
    // stays unported (see `extract_region_as_world`'s own doc comment).
    //
    // Two of the five exist to reach the `max(1, ...)` floor -- 10 km over a
    // 4/64 slice is 0.625 km, and 1 km over a 1/100 slice is 0.01 km. Without
    // them nothing distinguishes the floor from a plain multiply. A third is a
    // full-grid selection, where the km scale must come back UNCHANGED.
    #[allow(clippy::type_complexity)]
    let want: &[((usize, usize, f64), (usize, usize, usize, usize), usize, (usize, usize, f64))] = &[
        ((48, 32, 800.0), (4, 4, 24, 16), 1024, (1024, 683, 400.0)),
        ((2048, 1311, 800.0), (0, 0, 2048, 1311), 1024, (1024, 656, 800.0)),
        ((64, 64, 10.0), (60, 60, 4, 4), 256, (256, 256, 1.0)),
        ((100, 50, 1.0), (0, 0, 1, 1), 64, (64, 64, 1.0)),
        ((12, 9, 600.0), (1, 1, 6, 5), 128, (128, 107, 300.0)),
    ];
    for &((gw, gh, km), (x, y, w, h), ts, (want_gw, want_gh, want_km)) in want {
        let src = synthetic_field(gw, gh, 3);
        let plan = cartalith_engine::region_export::extract_region_as_world(
            &src, gw, gh, &Region { x, y, w, h }, ts, km,
            &AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() },
        );
        assert_eq!((plan.gw, plan.gh), (want_gw, want_gh), "{gw}x{gh} sel {w}x{h} ts {ts}");
        // Raw bit patterns: this is a scale every downstream km measurement
        // divides by, so "close enough" is not the bar.
        assert_eq!(plan.map_width_km.to_bits(), want_km.to_bits(), "newMapWidthKm for {gw}x{gh}");
        // Shape, asserted rather than assumed: a silently-empty or constant
        // extraction would satisfy every dimension check above.
        assert_eq!(plan.field.len(), want_gw * want_gh);
        assert!(plan.field.iter().all(|v| (0.0..=1.0).contains(v)), "stays in [0,1]");
        let first = plan.field[0];
        if w > 1 && h > 1 {
            assert!(plan.field.iter().any(|&v| v != first), "the extraction is not constant");
        } else {
            // A 1x1 selection genuinely IS constant -- `amplifyRegion`'s own
            // `rw > 1 ? ... : rx` collapses the coordinate mapping, which
            // milestone E pinned as the finite companion to its all-NaN case.
            assert!(plan.field.iter().all(|&v| v == first && v.is_finite()));
        }
    }
}

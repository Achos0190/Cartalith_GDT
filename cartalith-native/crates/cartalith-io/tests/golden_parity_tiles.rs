//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's region-export
//! encodings: `packHeight16`/`unpackHeight16` (reference lines 11544/11548)
//! and `buildTileManifest` (11554).
//!
//! Harness: the same Node `vm.runInContext` run of **whole `<script>` blocks**
//! documented in `cartalith-terrain/tests/golden_parity_amplify.rs` — blocks #1
//! (2084-14556) and #2 (14563-26720), delimiters asserted against the real
//! `<script>`/`</script>` tags, with the block-comment balance check (which
//! fired twice, wrongly, and was fixed rather than deleted — see that file).
//!
//! Every value below is compared **exactly**: packed bytes as an FNV-1a-64
//! over the raw buffer, manifests as full literal JSON strings. No tolerance
//! anywhere.
//!
//! The manifest comparison is byte-for-byte on purpose. `serde_json` writes an
//! `f64` of `16.0` as `16.0` where `JSON.stringify` writes `16`, and a
//! schema-2 manifest is a file other tools read — so `manifest_json` formats
//! numbers itself, and case 3 below is chosen specifically because `cols = 7`
//! does *not* divide `bounds.w = 30`: every `coarse.x`/`coarse.w` in it is a
//! long fraction, which is exactly where a shortest-round-trip formatter could
//! disagree with V8's. A fixture with only round numbers would have proved
//! nothing about that.

use cartalith_io::{
    build_tile_manifest, manifest_json, pack_height16, unpack_height16, CoarseBounds,
    TileManifestOpts,
};

fn fnv_u8(a: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in a {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

/// The harness's own `mkField`, arithmetic-for-arithmetic (see
/// `golden_parity_amplify.rs` for why it is synthetic and pure-arithmetic).
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
fn pack_height16_matches_the_reference_byte_for_byte() {
    // 0, 1, a half, both out-of-range clamps, a repeating fraction, and the
    // two values one 16-bit step from each end.
    let probe: Vec<f32> =
        vec![0.0, 1.0, 0.5, -0.25, 1.75, 1.0 / 3.0, 65534.0 / 65535.0, 1.0 / 65535.0, 0.9999999];
    let packed = pack_height16(&probe, probe.len());
    let want: Vec<u8> = vec![
        0, 0, 0, 255, 255, 255, 0, 255, 128, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0, 255, 85, 85, 0,
        255, 255, 254, 0, 255, 0, 1, 0, 255, 255, 255, 0, 255,
    ];
    assert_eq!(packed, want);
    assert_eq!(fnv_u8(&packed), "93de0af59a8a803a");
}

#[test]
fn unpack_height16_round_trips_exactly_as_the_reference_does() {
    let probe: Vec<f32> =
        vec![0.0, 1.0, 0.5, -0.25, 1.75, 1.0 / 3.0, 65534.0 / 65535.0, 1.0 / 65535.0, 0.9999999];
    let back = unpack_height16(&pack_height16(&probe, probe.len()), probe.len());
    // The reference's own values, including the two places the 16-bit step is
    // visible (0.5 -> 0.5000076…) and the two clamps.
    let want: Vec<f32> = vec![
        0.0,
        1.0,
        0.500_007_6,
        0.0,
        1.0,
        0.333_333_34,
        0.999_984_74,
        1.525_902_2e-5,
        1.0,
    ];
    for (i, (a, b)) in back.iter().zip(want.iter()).enumerate() {
        assert_eq!(a.to_bits(), b.to_bits(), "cell {i}: {a} vs {b}");
    }
}

#[test]
fn packing_a_whole_field_matches_the_reference() {
    let f = synthetic_field(48, 32, 5);
    // The world under the encoder, checked before trusting the encoding.
    let mut hf: u64 = 0xcbf2_9ce4_8422_2325;
    for v in &f {
        for &b in &v.to_bits().to_le_bytes() {
            hf ^= b as u64;
            hf = hf.wrapping_mul(0x100_0000_01b3);
        }
    }
    assert_eq!(format!("{hf:016x}"), "e6a8f7dd46187082");

    let packed = pack_height16(&f, f.len());
    assert_eq!(packed.len(), 6144);
    assert_eq!(fnv_u8(&packed), "0835250dc2d3fcb3");
}

#[test]
fn manifest_case0_a_full_refine_export_header() {
    let m = build_tile_manifest(
        &TileManifestOpts {
            cols: 2,
            rows: 3,
            tile_size: 512,
            tile_w: 512,
            tile_h: 256,
            width: 1024,
            height: 768,
            seed: 24601,
            world: true,
            bounds: Some(CoarseBounds { x: 4.0, y: 6.0, w: 30.0, h: 24.0 }),
            height_encoding: "rg16".into(),
            compression: "gzip".into(),
            version: "TESTVER".into(),
        },
        Some(&|r, c| format!("tiles/refined_{r}_{c}.png")),
    );
    assert_eq!(manifest_json(&m, None), r#"{"schema":2,"version":"TESTVER","tileSize":512,"tileW":512,"tileH":256,"cols":2,"rows":3,"width":1024,"height":768,"worldSeed":24601,"world":true,"bounds":{"x":4,"y":6,"w":30,"h":24},"heightEncoding":"rg16","compression":"gzip","tiles":[{"row":0,"col":0,"file":"tiles/refined_0_0.png","coarse":{"x":4,"y":6,"w":16,"h":9}},{"row":0,"col":1,"file":"tiles/refined_0_1.png","coarse":{"x":19,"y":6,"w":16,"h":9}},{"row":1,"col":0,"file":"tiles/refined_1_0.png","coarse":{"x":4,"y":14,"w":16,"h":9}},{"row":1,"col":1,"file":"tiles/refined_1_1.png","coarse":{"x":19,"y":14,"w":16,"h":9}},{"row":2,"col":0,"file":"tiles/refined_2_0.png","coarse":{"x":4,"y":22,"w":16,"h":9}},{"row":2,"col":1,"file":"tiles/refined_2_1.png","coarse":{"x":19,"y":22,"w":16,"h":9}}]}"#);
}

#[test]
fn manifest_case1_an_empty_bag_takes_every_fallback() {
    let m = build_tile_manifest(&TileManifestOpts { version: "TESTVER".into(), ..Default::default() }, None);
    // Note what is absent as much as what is present: no `coarse` key at all
    // when no bounds were given, and `bounds` itself is null rather than {}.
    assert_eq!(manifest_json(&m, None), r#"{"schema":2,"version":"TESTVER","tileSize":1024,"tileW":1024,"tileH":1024,"cols":1,"rows":1,"width":0,"height":0,"worldSeed":0,"world":false,"bounds":null,"heightEncoding":"none","compression":"store","tiles":[{"row":0,"col":0,"file":"tiles/tile_0_0.png"}]}"#);
}

#[test]
fn manifest_case2_a_whole_map_bake_with_no_coarse_region() {
    let m = build_tile_manifest(
        &TileManifestOpts {
            cols: 2,
            rows: 2,
            tile_size: 1024,
            width: 2048,
            height: 2048,
            seed: 7,
            version: "TESTVER".into(),
            ..Default::default()
        },
        None,
    );
    assert_eq!(manifest_json(&m, None), r#"{"schema":2,"version":"TESTVER","tileSize":1024,"tileW":1024,"tileH":1024,"cols":2,"rows":2,"width":2048,"height":2048,"worldSeed":7,"world":false,"bounds":null,"heightEncoding":"none","compression":"store","tiles":[{"row":0,"col":0,"file":"tiles/tile_0_0.png"},{"row":0,"col":1,"file":"tiles/tile_0_1.png"},{"row":1,"col":0,"file":"tiles/tile_1_0.png"},{"row":1,"col":1,"file":"tiles/tile_1_1.png"}]}"#);
}

#[test]
fn manifest_case3_fractional_coarse_bounds_and_a_negative_seed() {
    // cols=7 does not divide bounds.w=30, so every coarse x/w is a long
    // fraction -- the case that actually tests the number formatter.
    let m = build_tile_manifest(
        &TileManifestOpts {
            cols: 7,
            rows: 3,
            tile_size: 64,
            seed: -5,
            world: true,
            bounds: Some(CoarseBounds { x: 1.0, y: 2.0, w: 30.0, h: 20.0 }),
            height_encoding: "rg16".into(),
            version: "TESTVER".into(),
            ..Default::default()
        },
        None,
    );
    let json = manifest_json(&m, None);
    assert_eq!(fnv_u8(json.as_bytes()), "cf449deddeb5e94c");
    assert_eq!(json.len(), 3132);
    // The two fractions spelled out, so a formatter regression names itself
    // rather than only moving the hash.
    assert!(json.contains(r#""w":5.285714285714286"#), "{json}");
    assert!(json.contains(r#""h":7.666666666666667"#));
    assert!(json.contains(r#""y":8.666666666666668"#));
    assert!(json.contains(r#""worldSeed":-5"#));
}

#[test]
fn the_pretty_index_json_matches_export_region_tiles_byte_for_byte() {
    // What `exportRegionTiles` actually writes into `tiles/index.json`:
    // JSON.stringify(man, null, 2) for the 2x2 refine of a 24x16 selection.
    let m = build_tile_manifest(
        &TileManifestOpts {
            cols: 2,
            rows: 2,
            tile_size: 32,
            tile_w: 32,
            tile_h: 21,
            width: 64,
            height: 42,
            seed: 4242,
            world: false,
            bounds: Some(CoarseBounds { x: 4.0, y: 4.0, w: 24.0, h: 16.0 }),
            height_encoding: "rg16".into(),
            compression: "store".into(),
            version: "TESTVER".into(),
        },
        Some(&|r, c| format!("tiles/refined_{r}_{c}.png")),
    );
    let json = manifest_json(&m, Some(2));
    assert_eq!(fnv_u8(json.as_bytes()), "a2c757b803ca92ad");
    assert_eq!(json.len(), 1024);
    assert!(json.starts_with("{\n  \"schema\": 2,\n  \"version\": \"TESTVER\",\n"));
    assert!(json.contains("  \"bounds\": {\n    \"x\": 4,\n    \"y\": 4,\n    \"w\": 24,\n    \"h\": 16\n  },\n"));
    assert!(json.contains("  \"tiles\": [\n    {\n      \"row\": 0,\n      \"col\": 0,\n"));
    assert!(json.ends_with("\n}"));
}

//! Golden-parity tests for the bake's composed half — `pyramidTile`
//! (reference line 10575), `atlasEncodeChunk` (10712) and
//! `buildAtlasManifest` (10882).
//!
//! # The harness
//!
//! The same one `cartalith-spatial`'s `golden_parity_pyramid.rs` documents in
//! full: Node `vm.runInContext` over the **whole** `<script>` block #1
//! (2084-14556), both delimiters asserted, the probe appended to the block's
//! own source so it shares the block's `let` scope (`GW`/`GH`/`state`/`VERSION`
//! are lexical bindings, not `vm` context properties — `CLAUDE.md`'s own
//! documented hazard), and a truthy `indexedDB` stub so block #1's boot line
//! does not auto-generate a 2048-wide world before the probe runs.
//!
//! # Why this is a separate file from the two below it
//!
//! `pyramidTile` is *composition* — `tileDims` × `refineTile` ×
//! `addZoomDetail` — and each of those three is golden-verified where it lives
//! (`cartalith-spatial`'s `golden_parity_region.rs`,
//! `cartalith-terrain`'s `golden_parity_amplify.rs` and
//! `golden_parity_zoom_detail.rs`). Verifying the composition separately is
//! what upgrades "the three parts match" to "and they are wired together in
//! the reference's order, with the reference's arguments" — the same
//! distinction `region_export.rs`'s milestone-E2 note draws.
//!
//! # Emptiness and shape assertions
//!
//! Per `CLAUDE.md`, the extraction asserted before any golden was written
//! down: that every tile's length is exactly `w*h`, that **no tile is
//! constant** (a composition bug that fed every tile the same collapsed
//! sub-region would still produce a full, well-formed atlas), and that two
//! horizontally adjacent tiles agree on their shared edge with delta **exactly
//! zero**. All are re-asserted here.

use cartalith_engine::bake::pyramid_tile;
use cartalith_io::atlas::{build_atlas_manifest, encode_chunk, AtlasChunkDesc};
use cartalith_spatial::pyramid::ChunkId;
use cartalith_terrain::amplify::AmplifyOpts;

fn fnv_f32(a: &[f32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in a {
        for &b in &v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

fn fnv_u8(a: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in a {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

/// The harness's own `mkField`, arithmetic for arithmetic.
fn synthetic_field(cw: usize, ch: usize, k: i64) -> Vec<f32> {
    let mut f = vec![0.0f32; cw * ch];
    let (cx, cy) = (cw as f64 * 0.42, ch as f64 * 0.55);
    let r2 = (cw as f64 * 0.3) * (ch as f64 * 0.3);
    for y in 0..ch {
        for x in 0..cw {
            let (dx, dy) = (x as f64 - cx, y as f64 - cy);
            let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
            let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
            v += 0.05 * ((q as f64 / 10.0) - 0.5);
            v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - ch as f64 * 0.25).abs() / (ch as f64 * 0.12));
            f[y * cw + x] = v.clamp(0.0, 1.0) as f32;
        }
    }
    f
}

const CW: usize = 48;
const CH: usize = 32;

fn opts() -> AmplifyOpts {
    AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, detail_freq: 1.0, ..Default::default() }
}

#[test]
fn the_fixture_is_bit_identical_to_the_harnesss() {
    let f = synthetic_field(CW, CH, 5);
    assert_eq!(fnv_f32(&f), "e6a8f7dd46187082", "the fixture itself diverged");
}

#[test]
fn pyramid_tile_matches_the_reference() {
    // (z, col, row, tileSize) -> (w, h, hash, min, max).
    let cases: &[(u32, u32, u32, usize, usize, usize, &str, f32, f32)] = &[
        (0, 0, 0, 64, 64, 42, "d1784219c7e13824", 0.2750000059604645, 0.9290775656700134),
        (1, 0, 0, 64, 64, 42, "02508e5c6b040cb6", 0.2752439081668854, 0.9095419645309448),
        (1, 1, 1, 64, 64, 42, "4e49d93d747b6506", 0.2750000059604645, 0.8792386651039124),
        (2, 2, 1, 32, 32, 21, "9b4fa390b647d604", 0.2776673436164856, 0.8528468012809753),
        // z = 3 is the first level `addZoomDetail` actually fires on (zBase 2),
        // so this case is the one that proves the composition, not just the
        // refine.
        (3, 5, 3, 32, 32, 21, "c607e279b67a2d8c", 0.2761129140853882, 0.5274623036384583),
        // Two deep levels, added after mutation testing: at z <= 3 the zoom
        // detail contributes at most **one** octave, so a constant governing
        // the *second and later* ones (`f *= 2`, `amp *= 0.6`, the six-octave
        // cap) is invisible to every case above -- three such mutants survived
        // this file while `golden_parity_zoom_detail.rs` killed all three.
        // z = 5 reaches three octaves and z = 7 reaches five.
        (5, 17, 11, 32, 32, 21, "e6b2e6a4b68b2c26", 0.5378698706626892, 0.6654000878334045),
        (7, 80, 50, 16, 16, 11, "911bfe659c5ade2c", 0.3799262046813965, 0.412759929895401),
    ];
    let f = synthetic_field(CW, CH, 5);
    for &(z, col, row, ts, w, h, hash, min, max) in cases {
        let t = pyramid_tile(&f, CW, CH, ChunkId::new(z, col, row), ts, &opts());
        assert_eq!((t.w, t.h), (w, h), "z={z} col={col} row={row} dims");
        assert_eq!(t.data.len(), w * h, "z={z} length");
        assert!(t.data.iter().any(|&v| v != t.data[0]), "z={z} tile is constant");
        assert_eq!(t.data.iter().copied().fold(f32::INFINITY, f32::min), min, "z={z} min");
        assert_eq!(t.data.iter().copied().fold(f32::NEG_INFINITY, f32::max), max, "z={z} max");
        assert_eq!(fnv_f32(&t.data), hash, "z={z} col={col} row={row}");
    }
}

#[test]
fn pyramid_tiles_first_six_samples_match_the_reference() {
    // Hashes prove agreement but say nothing when they disagree. These are the
    // first six values of the level-0 tile, so a failure points at a texel.
    let f = synthetic_field(CW, CH, 5);
    let t = pyramid_tile(&f, CW, CH, ChunkId::new(0, 0, 0), 64, &opts());
    let want: [f32; 6] = [
        0.300_000_011_920_928_96,
        0.285_079_360_008_239_75,
        0.297_222_226_858_139_04,
        0.310_238_093_137_741_1,
        0.295_317_441_225_051_9,
        0.280_396_819_114_685_06,
    ];
    assert_eq!(&t.data[..6], &want[..]);
}

#[test]
fn adjacent_tiles_seam_delta_is_exactly_zero() {
    // The harness measured 0, not "small". Anything else is a hairline down
    // every tile boundary in the assembled pyramid.
    let f = synthetic_field(CW, CH, 5);
    let a = pyramid_tile(&f, CW, CH, ChunkId::new(2, 1, 1), 32, &opts());
    let b = pyramid_tile(&f, CW, CH, ChunkId::new(2, 2, 1), 32, &opts());
    let maxd = (0..a.h)
        .map(|y| (a.data[y * a.w + (a.w - 1)] - b.data[y * b.w]).abs())
        .fold(0.0f32, f32::max);
    assert_eq!(maxd, 0.0);
}

#[test]
fn atlas_encode_chunk_matches_the_reference() {
    let f = synthetic_field(CW, CH, 5);
    let t = pyramid_tile(&f, CW, CH, ChunkId::new(1, 1, 0), 32, &opts());
    assert_eq!(fnv_f32(&t.data), "66660c14fbef99ca", "the tile fed to the encoder diverged");
    let c = encode_chunk(t.id, &t.data, t.w, t.h, None);
    assert_eq!(c.rg16.len(), 2688, "32 x 21 x 4 bytes");
    assert_eq!(fnv_u8(&c.rg16), "2e9f14812814176c");
}

#[test]
fn build_atlas_manifest_matches_the_reference_byte_for_byte() {
    // `JSON.stringify(m, null, 2)` from the harness. serde_json's pretty
    // printer agrees on every value here because they are all integers,
    // strings, bools and null -- see `AtlasManifest`'s own note on why the
    // *tile* manifest needed a hand-rolled writer and this one does not.
    let chunks = [
        AtlasChunkDesc { id: ChunkId::new(0, 0, 0), w: 64, h: 43, gzip: false, png: true },
        AtlasChunkDesc { id: ChunkId::new(1, 1, 0), w: 64, h: 43, gzip: true, png: false },
    ];
    let m = build_atlas_manifest("41a0d664", &chunks, 1024, "2.10", 1_700_000_000_000, None);
    let got = serde_json::to_string_pretty(&m).expect("serialises");
    let want = r#"{
  "schema": 1,
  "kind": "cartalith-atlas",
  "worldKey": "41a0d664",
  "version": "2.10",
  "tileSize": 1024,
  "time": 1700000000000,
  "count": 2,
  "params": null,
  "chunks": [
    {
      "z": 0,
      "col": 0,
      "row": 0,
      "w": 64,
      "h": 43,
      "bin": "World/LOD0/0_0_0.bin",
      "png": "World/LOD0/0_0_0.png",
      "gzip": false
    },
    {
      "z": 1,
      "col": 1,
      "row": 0,
      "w": 64,
      "h": 43,
      "bin": "World/LOD1/1_1_0.bin.gz",
      "png": null,
      "gzip": true
    }
  ]
}"#;
    assert_eq!(got, want);
}

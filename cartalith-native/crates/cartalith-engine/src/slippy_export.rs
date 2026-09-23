//! A slippy-map tile pyramid export — the LOD pyramid written out under
//! XYZ, TMS or WMTS addressing, with optional `@Nx` retina variants.
//!
//! Composition only, like [`crate::region_export`]: every tile is
//! [`crate::bake::pyramid_tile`] (the bake's own synthesis, so an exported
//! `z/x/y` is byte-for-byte the tile a bake stores at that address) rendered
//! by [`crate::region_export::tile_png_bytes`], and named by
//! [`cartalith_io::slippy`]. See that module for what the three schemes are
//! and what the manifest does and does not claim.
//!
//! **Always synthesises; never reads the atlas.** The canvas's estimate says
//! *"source baked atlas · no re-gen"*, but a stored chunk's PNG was shaded
//! under the sun/exaggeration of the moment it was baked, and those are
//! presentation settings the world key does not hash — so a cached PNG can be
//! stale while its key is current. Synthesis is deterministic, so re-running
//! it costs time and cannot cost correctness.

use cartalith_io::slippy::{slippy_manifest, tile_path, zoom_ladder, TileScheme, SLIPPY_MANIFEST};
use cartalith_spatial::pyramid::{pyramid_dims, ChunkId};
use cartalith_terrain::amplify::AmplifyOpts;
use rayon::prelude::*;

use crate::bake::pyramid_tile;
use crate::region_export::{tile_png_bytes, RegionTileEntry, TileVisual};

/// Everything a slippy export needs besides the field.
#[derive(Debug, Clone)]
pub struct SlippyExportOpts<'a> {
    pub scheme: TileScheme,
    /// Deepest level written; levels `0..=max_z` all are.
    pub max_z: u32,
    /// Long edge of a scale-1 tile, in pixels.
    pub tile_size: usize,
    /// Pixel-density variants to write; `1` is the plain tile. `[1, 2]` is
    /// the usual "with retina".
    pub scales: &'a [u32],
    pub amplify: &'a AmplifyOpts,
    pub visual: TileVisual,
    /// For the manifest's ladder only.
    pub map_width_km: f64,
    pub name: &'a str,
    pub version: &'a str,
}

/// The result of one slippy export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlippyExport {
    /// Tiles level by level, row-major within a level, scale by scale; then
    /// the manifest ([`SLIPPY_MANIFEST`]) last.
    pub entries: Vec<RegionTileEntry>,
    /// Tiles whose PNG encode failed and were therefore not written. Non-zero
    /// is a real hole in the pyramid the caller must surface.
    pub failed: usize,
}

/// Tiles one export writes: `scales × (4^(max_z+1) − 1) / 3`.
pub fn slippy_tile_count(max_z: u32, scales: usize) -> u64 {
    cartalith_spatial::pyramid::pyramid_tile_count(max_z as i32) * scales as u64
}

/// Write the pyramid. Synthesis is `rayon`-parallel within a level; output
/// order is deterministic regardless.
///
/// # Panics
///
/// [`pyramid_tile`]'s: `coarse` shorter than `cw * ch`, or a zero dimension.
pub fn export_slippy_tiles(coarse: &[f32], cw: usize, ch: usize, o: &SlippyExportOpts<'_>) -> SlippyExport {
    let mut entries = Vec::new();
    let mut failed = 0usize;
    for &scale in o.scales {
        let ts = o.tile_size * scale.max(1) as usize;
        for z in 0..=o.max_z {
            let d = pyramid_dims(z as i32);
            let ids: Vec<ChunkId> =
                (0..d.rows).flat_map(|r| (0..d.cols).map(move |c| ChunkId::new(z, c, r))).collect();
            let pngs: Vec<Option<Vec<u8>>> = ids
                .par_iter()
                .map(|&id| {
                    let t = pyramid_tile(coarse, cw, ch, id, ts, o.amplify);
                    tile_png_bytes(&t.data, t.w, t.h, &o.visual)
                })
                .collect();
            for (id, png) in ids.into_iter().zip(pngs) {
                match png {
                    Some(data) => entries.push(RegionTileEntry { name: tile_path(o.scheme, id, scale, "png"), data }),
                    None => failed += 1,
                }
            }
        }
    }
    let ladder = zoom_ladder(cw, ch, o.map_width_km, o.tile_size, o.max_z);
    entries.push(RegionTileEntry {
        name: SLIPPY_MANIFEST.into(),
        data: slippy_manifest(o.scheme, &ladder, o.scales, o.name, o.version).into_bytes(),
    });
    SlippyExport { entries, failed }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(gw: usize, gh: usize) -> Vec<f32> {
        (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f32, (i / gw) as f32);
                (0.3 + 0.5 * ((x * 0.37).sin() * (y * 0.23).cos()).abs()).clamp(0.0, 1.0)
            })
            .collect()
    }

    fn amp() -> AmplifyOpts {
        AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, ..Default::default() }
    }

    fn run(scheme: TileScheme, scales: &[u32]) -> SlippyExport {
        let f = field(33, 17);
        let a = amp();
        export_slippy_tiles(&f, 33, 17, &SlippyExportOpts {
            scheme,
            max_z: 2,
            tile_size: 32,
            scales,
            amplify: &a,
            visual: TileVisual::default(),
            map_width_km: 330.0,
            name: "test",
            version: "TESTVER",
        })
    }

    #[test]
    fn every_tile_of_every_level_is_written_once_plus_the_manifest() {
        let e = run(TileScheme::Xyz, &[1]);
        assert_eq!(e.failed, 0);
        // 1 + 4 + 16 = 21 tiles, a literal rather than the count function.
        assert_eq!(e.entries.len(), 22);
        assert_eq!(e.entries.last().unwrap().name, "tiles.json");
        let mut names: Vec<&str> = e.entries.iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 22, "an address was written twice");
        assert!(names.contains(&"2/3/0.png") && names.contains(&"2/0/3.png"));
        assert_eq!(slippy_tile_count(2, 1), 21);
    }

    #[test]
    fn an_xyz_tile_is_the_bakes_tile_at_that_address() {
        // The pyramid must not be re-derived: XYZ 2/3/1 is pyramid_tile(z2, col3, row1).
        let e = run(TileScheme::Xyz, &[1]);
        let got = e.entries.iter().find(|t| t.name == "2/3/1.png").expect("2/3/1");
        let f = field(33, 17);
        let t = pyramid_tile(&f, 33, 17, ChunkId::new(2, 3, 1), 32, &amp());
        assert_eq!(got.data, tile_png_bytes(&t.data, t.w, t.h, &TileVisual::default()).unwrap());
    }

    #[test]
    fn tms_and_xyz_hold_the_same_bytes_under_flipped_rows() {
        let x = run(TileScheme::Xyz, &[1]);
        let t = run(TileScheme::Tms, &[1]);
        let get = |e: &SlippyExport, n: &str| e.entries.iter().find(|t| t.name == n).unwrap().data.clone();
        // Level 2 has 4 rows: XYZ row 0 is TMS row 3.
        assert_eq!(get(&x, "2/1/0.png"), get(&t, "2/1/3.png"));
        assert_ne!(get(&x, "2/1/0.png"), get(&t, "2/1/0.png"), "the flip must actually move a tile");
    }

    #[test]
    fn wmts_addresses_row_before_column() {
        let x = run(TileScheme::Xyz, &[1]);
        let w = run(TileScheme::Wmts, &[1]);
        let get = |e: &SlippyExport, n: &str| e.entries.iter().find(|t| t.name == n).unwrap().data.clone();
        assert_eq!(get(&x, "2/3/1.png"), get(&w, "cartalith/2/1/3.png"));
    }

    #[test]
    fn a_retina_variant_is_the_same_ground_at_twice_the_pixels() {
        let e = run(TileScheme::Xyz, &[1, 2]);
        assert_eq!(e.entries.len(), 43);
        let dec = |n: &str| {
            let t = e.entries.iter().find(|t| t.name == n).unwrap_or_else(|| panic!("{n}"));
            cartalith_assets::raster::decode_png(&t.data).expect("png")
        };
        let (a, b) = (dec("1/1/0.png"), dec("1/1/0@2x.png"));
        // 33x17 world, inset 32x16: 2:1 tiles, 32x16 then 64x32.
        assert_eq!((a.w, a.h), (32, 16));
        assert_eq!((b.w, b.h), (64, 32));
    }

    #[test]
    fn the_same_export_is_the_same_bytes_twice() {
        assert_eq!(run(TileScheme::Tms, &[1, 2]), run(TileScheme::Tms, &[1, 2]));
    }
}

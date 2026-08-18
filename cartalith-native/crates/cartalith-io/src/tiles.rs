//! Region-export encodings — `UNIFIED_TOOL_PLAN.md` milestone E, the
//! *file-format* half of the Region select/export tool.
//!
//! Ports `packHeight16`/`unpackHeight16` (reference lines 11544/11548) and
//! `buildTileManifest` (11554), the three pieces of `exportRegionTiles` that
//! are pure bytes rather than browser API calls. They live in `cartalith-io`
//! for the same reason `load_save` does: this crate owns what a Cartalith file
//! looks like on disk. The height *math* they encode lives in
//! `cartalith_terrain::amplify`; the composition of the two is
//! `cartalith_engine::region_export`.
//!
//! # Why the JSON is written by hand
//!
//! `serde_json` renders an `f64` of `16.0` as `16.0`; `JSON.stringify` renders
//! it as `16`. The manifest's `coarse` bounds are computed as
//! `bounds.w / cols`, which is an integer for most tile grids and a fraction
//! for the rest — so a serde round-trip would differ from the reference's own
//! output on exactly the common case, and the schema-2 manifest is a file
//! other tools read. [`manifest_json`] therefore formats numbers the way
//! `Number.prototype.toString` does. The [`TileManifest`] struct itself is
//! plain data and still derives `Serialize`/`Deserialize` for anyone who wants
//! a Rust-native round-trip instead.
//!
//! # Not ported here, deliberately
//!
//! The rest of `exportRegionTiles` — per-tile PNG rendering (`tilePngBytes`),
//! `gzipBytes`' `CompressionStream`, and the `.zip` assembly — is milestone
//! **E2** (see `UNIFIED_TOOL_PLAN.md`, "Milestone E as built"). So is
//! `exportGeoJSON`. Both are real, sizeable, and *format* work rather than
//! *geometry* work; splitting them out is the same honest boundary Journey
//! Planner M5 and urban M3 both drew.

use serde::{Deserialize, Serialize};

/// `packHeight16(fld, n)` (reference line 11544).
///
/// 16-bit height packed into an RGBA byte quad: `H = R·256 + G` over `[0,1]`,
/// `B = 0`, `A = 255`. The reference's own note: *"The `.f32` export already
/// round-trips full float; this is the engine-friendly 16-bit option the plan
/// calls for (canvas PNG is only 8-bit per channel). Pure → testable
/// round-trip."*
///
/// `n` bounds the run; pass `fld.len()` for the whole field. NaN packs to
/// zero, because `Math.round(NaN)` is `NaN` and JS's `>>`/`&` coerce that to
/// `0` — ported rather than rejected, since `amplify_region` can legitimately
/// produce NaN (see its `out_w == 1` note) and the reference would have
/// written zeroes there.
///
/// # Panics
///
/// Panics if `n > fld.len()`.
pub fn pack_height16(fld: &[f32], n: usize) -> Vec<u8> {
    assert!(n <= fld.len(), "pack_height16 asked for {n} cells of a {}-cell field", fld.len());
    let mut out = vec![0u8; n * 4];
    for i in 0..n {
        // `v<0?0:v>1?1:v`, which `f64::clamp` reproduces exactly, NaN and all.
        let v = (fld[i] as f64).clamp(0.0, 1.0);
        // `Math.round` is floor(x + 0.5); NaN survives it and then ToInt32s to 0.
        let rounded = (v * 65535.0 + 0.5).floor();
        let q: i64 = if rounded.is_finite() { rounded as i64 } else { 0 };
        out[i * 4] = ((q >> 8) & 255) as u8;
        out[i * 4 + 1] = (q & 255) as u8;
        out[i * 4 + 2] = 0;
        out[i * 4 + 3] = 255;
    }
    out
}

/// `unpackHeight16(rgba, n)` (reference line 11548) — the inverse of
/// [`pack_height16`], and the fallback `loadZip` uses when a save carries
/// `heightmap_rg16.bin` instead of `heightmap.f32`.
///
/// # Panics
///
/// Panics if `rgba` holds fewer than `n * 4` bytes.
pub fn unpack_height16(rgba: &[u8], n: usize) -> Vec<f32> {
    assert!(rgba.len() >= n * 4, "unpack_height16 needs {} bytes, got {}", n * 4, rgba.len());
    (0..n)
        .map(|i| ((((rgba[i * 4] as u32) << 8) | rgba[i * 4 + 1] as u32) as f64 / 65535.0) as f32)
        .collect()
}

/// A rectangle in the manifest's coarse-cell coordinates. Fractional by
/// construction: `bounds.w / cols` is rarely a whole number.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CoarseBounds {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// One tile's record in the manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRecord {
    pub row: usize,
    pub col: usize,
    pub file: String,
    /// Present only when the grid refines a coarse region (`bounds` set).
    pub coarse: Option<CoarseBounds>,
}

/// The schema-2 tile manifest (`tiles/index.json`).
///
/// The reference's own note: *"Superset of the old flat index
/// (`{tileSize,cols,rows,width,height}`) so existing consumers keep working;
/// adds schema, worldSeed/world, height encoding, compression, and per-tile
/// records (with coarse bounds when the grid refines a coarse region)."*
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileManifest {
    pub schema: u32,
    pub version: String,
    pub tile_size: usize,
    pub tile_w: usize,
    pub tile_h: usize,
    pub cols: usize,
    pub rows: usize,
    pub width: usize,
    pub height: usize,
    pub world_seed: i32,
    pub world: bool,
    pub bounds: Option<CoarseBounds>,
    pub height_encoding: String,
    pub compression: String,
    pub tiles: Vec<TileRecord>,
}

/// `buildTileManifest`'s `o` bag.
///
/// Every numeric field follows the reference's `o.x || fallback` spelling, so
/// **zero means "use the default"** exactly as it does in JS — `cols: 0` gives
/// 1, `tile_size: 0` gives 1024, `tile_w: 0` gives `tile_size`. Reproduced
/// rather than tidied: a caller that passes 0 today gets the fallback, and
/// "fixing" it would silently start emitting a zero-column manifest.
/// `width`/`height`/`seed` fall back to 0, which is the same value, so they
/// are plain fields.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TileManifestOpts {
    pub cols: usize,
    pub rows: usize,
    pub tile_size: usize,
    pub tile_w: usize,
    pub tile_h: usize,
    pub width: usize,
    pub height: usize,
    pub seed: i32,
    pub world: bool,
    pub bounds: Option<CoarseBounds>,
    /// `''` becomes `"none"`, matching `o.heightEncoding || 'none'`.
    pub height_encoding: String,
    /// `''` becomes `"store"`.
    pub compression: String,
    /// The reference falls back to its own `VERSION` global here. This port has
    /// no app-version constant in a library crate, so the caller supplies it.
    pub version: String,
}

/// `buildTileManifest(o)` (reference line 11554).
///
/// `file_for` is the reference's `o.fileFor`; `None` uses its own default
/// `tiles/tile_{row}_{col}.png`.
pub fn build_tile_manifest(
    o: &TileManifestOpts,
    file_for: Option<&dyn Fn(usize, usize) -> String>,
) -> TileManifest {
    let cols = if o.cols == 0 { 1 } else { o.cols };
    let rows = if o.rows == 0 { 1 } else { o.rows };
    let ts = if o.tile_size == 0 { 1024 } else { o.tile_size };
    let has_coarse = o.bounds.is_some();
    let (step_x, step_y) = match o.bounds {
        Some(b) => (b.w / cols as f64, b.h / rows as f64),
        None => (0.0, 0.0),
    };
    let mut tiles = Vec::with_capacity(cols * rows);
    for r in 0..rows {
        for c in 0..cols {
            let file = match file_for {
                Some(f) => f(r, c),
                None => format!("tiles/tile_{r}_{c}.png"),
            };
            let coarse = if has_coarse {
                let b = o.bounds.expect("has_coarse implies bounds");
                Some(CoarseBounds {
                    x: b.x + c as f64 * step_x,
                    y: b.y + r as f64 * step_y,
                    w: step_x + 1.0,
                    h: step_y + 1.0,
                })
            } else {
                None
            };
            tiles.push(TileRecord { row: r, col: c, file, coarse });
        }
    }
    TileManifest {
        schema: 2,
        version: o.version.clone(),
        tile_size: ts,
        tile_w: if o.tile_w == 0 { ts } else { o.tile_w },
        tile_h: if o.tile_h == 0 { ts } else { o.tile_h },
        cols,
        rows,
        width: o.width,
        height: o.height,
        world_seed: o.seed,
        world: o.world,
        bounds: o.bounds,
        height_encoding: if o.height_encoding.is_empty() {
            "none".to_string()
        } else {
            o.height_encoding.clone()
        },
        compression: if o.compression.is_empty() {
            "store".to_string()
        } else {
            o.compression.clone()
        },
        tiles,
    }
}

/// `Number.prototype.toString` for the values a manifest can hold.
///
/// The only place Rust and JS disagree in this range is the integral case:
/// `format!("{}", 16.0_f64)` and `Number(16).toString()` both give `16`, but
/// going through `serde_json` gives `16.0`. Spelled out explicitly rather than
/// relying on `Display`'s current behaviour, since the manifest is a file
/// other tools parse.
pub fn js_num(v: f64) -> String {
    if v.is_nan() {
        return "null".to_string(); // JSON.stringify(NaN) === "null"
    }
    if !v.is_finite() {
        return "null".to_string();
    }
    if v == v.trunc() && v.abs() < 1e21 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// A JSON string literal, escaped the way `JSON.stringify` escapes one.
///
/// Backspace and form feed get their short forms rather than ``/``,
/// matching V8's own `QuoteJSONString` table. Shared with the GeoJSON writer in
/// `cartalith-engine`, where a place name is arbitrary user text rather than a
/// manifest key.
pub fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Render a manifest exactly as `JSON.stringify` would.
///
/// `indent` of `None` is the compact form; `Some(n)` matches
/// `JSON.stringify(man, null, n)`, which is what `exportRegionTiles` writes
/// into `tiles/index.json` (with `n = 2`).
pub fn manifest_json(m: &TileManifest, indent: Option<usize>) -> String {
    let mut s = String::new();
    let w = Writer { indent };
    w.object(&mut s, 0, &[
        ("schema", Val::Num(m.schema as f64)),
        ("version", Val::Str(&m.version)),
        ("tileSize", Val::Num(m.tile_size as f64)),
        ("tileW", Val::Num(m.tile_w as f64)),
        ("tileH", Val::Num(m.tile_h as f64)),
        ("cols", Val::Num(m.cols as f64)),
        ("rows", Val::Num(m.rows as f64)),
        ("width", Val::Num(m.width as f64)),
        ("height", Val::Num(m.height as f64)),
        ("worldSeed", Val::Num(m.world_seed as f64)),
        ("world", Val::Bool(m.world)),
        ("bounds", match &m.bounds {
            Some(b) => Val::Bounds(*b),
            None => Val::Null,
        }),
        ("heightEncoding", Val::Str(&m.height_encoding)),
        ("compression", Val::Str(&m.compression)),
        ("tiles", Val::Tiles(&m.tiles)),
    ]);
    s
}

enum Val<'a> {
    Num(f64),
    Str(&'a str),
    Bool(bool),
    Null,
    Bounds(CoarseBounds),
    Tiles(&'a [TileRecord]),
}

struct Writer {
    indent: Option<usize>,
}

impl Writer {
    fn pad(&self, out: &mut String, depth: usize) {
        if let Some(n) = self.indent {
            out.push('\n');
            for _ in 0..n * depth {
                out.push(' ');
            }
        }
    }
    fn colon(&self) -> &'static str {
        if self.indent.is_some() {
            ": "
        } else {
            ":"
        }
    }

    fn object(&self, out: &mut String, depth: usize, fields: &[(&str, Val)]) {
        out.push('{');
        for (i, (k, v)) in fields.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            self.pad(out, depth + 1);
            out.push_str(&json_string(k));
            out.push_str(self.colon());
            self.value(out, depth + 1, v);
        }
        if !fields.is_empty() {
            self.pad(out, depth);
        }
        out.push('}');
    }

    fn value(&self, out: &mut String, depth: usize, v: &Val) {
        match v {
            Val::Num(n) => out.push_str(&js_num(*n)),
            Val::Str(s) => out.push_str(&json_string(s)),
            Val::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Val::Null => out.push_str("null"),
            Val::Bounds(b) => self.object(out, depth, &[
                ("x", Val::Num(b.x)),
                ("y", Val::Num(b.y)),
                ("w", Val::Num(b.w)),
                ("h", Val::Num(b.h)),
            ]),
            Val::Tiles(ts) => {
                out.push('[');
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    self.pad(out, depth + 1);
                    let mut fields: Vec<(&str, Val)> = vec![
                        ("row", Val::Num(t.row as f64)),
                        ("col", Val::Num(t.col as f64)),
                        ("file", Val::Str(&t.file)),
                    ];
                    if let Some(c) = t.coarse {
                        fields.push(("coarse", Val::Bounds(c)));
                    }
                    self.object(out, depth + 1, &fields);
                }
                if !ts.is_empty() {
                    self.pad(out, depth);
                }
                out.push(']');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packing_uses_four_bytes_per_cell_with_a_constant_blue_and_alpha() {
        let p = pack_height16(&[0.5, 0.25], 2);
        assert_eq!(p.len(), 8);
        assert_eq!(p[2], 0);
        assert_eq!(p[3], 255);
        assert_eq!(p[6], 0);
        assert_eq!(p[7], 255);
    }

    #[test]
    fn zero_and_one_pack_to_the_ends_of_the_range() {
        let p = pack_height16(&[0.0, 1.0], 2);
        assert_eq!(&p[0..2], &[0, 0]);
        assert_eq!(&p[4..6], &[255, 255]);
    }

    #[test]
    fn values_outside_the_unit_range_are_clamped_not_wrapped() {
        let p = pack_height16(&[-0.25, 1.75], 2);
        assert_eq!(&p[0..2], &[0, 0]);
        assert_eq!(&p[4..6], &[255, 255]);
    }

    #[test]
    fn nan_packs_to_zero_the_way_js_coerces_it() {
        let p = pack_height16(&[f32::NAN], 1);
        assert_eq!(&p[0..4], &[0, 0, 0, 255]);
    }

    #[test]
    fn the_round_trip_is_accurate_to_the_16_bit_step() {
        let src: Vec<f32> = (0..64).map(|i| i as f32 / 63.0).collect();
        let back = unpack_height16(&pack_height16(&src, src.len()), src.len());
        for (a, b) in src.iter().zip(back.iter()) {
            assert!((a - b).abs() <= 1.0 / 65535.0, "{a} vs {b}");
        }
    }

    #[test]
    fn unpack_reads_the_high_byte_first() {
        // R=1, G=0 is 256/65535, not 1/65535.
        let v = unpack_height16(&[1, 0, 0, 255], 1);
        assert!((v[0] as f64 - 256.0 / 65535.0).abs() < 1e-9);
    }

    #[test]
    #[should_panic(expected = "pack_height16 asked for")]
    fn packing_past_the_end_of_the_field_is_rejected() {
        pack_height16(&[0.0], 4);
    }

    #[test]
    #[should_panic(expected = "unpack_height16 needs")]
    fn unpacking_past_the_end_of_the_buffer_is_rejected() {
        unpack_height16(&[0, 0, 0, 255], 4);
    }

    fn opts() -> TileManifestOpts {
        TileManifestOpts { version: "TESTVER".into(), ..Default::default() }
    }

    #[test]
    fn an_empty_bag_yields_the_references_own_defaults() {
        let m = build_tile_manifest(&opts(), None);
        assert_eq!(m.schema, 2);
        assert_eq!((m.cols, m.rows), (1, 1));
        assert_eq!(m.tile_size, 1024);
        assert_eq!((m.tile_w, m.tile_h), (1024, 1024));
        assert_eq!(m.height_encoding, "none");
        assert_eq!(m.compression, "store");
        assert!(m.bounds.is_none());
        assert_eq!(m.tiles.len(), 1);
        assert_eq!(m.tiles[0].file, "tiles/tile_0_0.png");
        assert!(m.tiles[0].coarse.is_none());
    }

    #[test]
    fn a_zero_column_count_means_one_column_like_the_js_or_fallback() {
        let m = build_tile_manifest(&TileManifestOpts { cols: 0, rows: 0, ..opts() }, None);
        assert_eq!((m.cols, m.rows), (1, 1));
    }

    #[test]
    fn tile_dims_fall_back_to_the_tile_size_when_absent() {
        let m = build_tile_manifest(&TileManifestOpts { tile_size: 256, ..opts() }, None);
        assert_eq!((m.tile_w, m.tile_h), (256, 256));
    }

    #[test]
    fn tiles_are_emitted_row_major() {
        let m = build_tile_manifest(&TileManifestOpts { cols: 3, rows: 2, ..opts() }, None);
        let order: Vec<(usize, usize)> = m.tiles.iter().map(|t| (t.row, t.col)).collect();
        assert_eq!(order, vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)]);
    }

    #[test]
    fn coarse_bounds_appear_only_when_a_region_was_given_and_overlap_by_one_cell() {
        let b = CoarseBounds { x: 4.0, y: 6.0, w: 30.0, h: 24.0 };
        let m = build_tile_manifest(&TileManifestOpts { cols: 2, rows: 3, bounds: Some(b), ..opts() }, None);
        let c0 = m.tiles[0].coarse.expect("tile 0 coarse");
        assert_eq!((c0.x, c0.y), (4.0, 6.0));
        // stepX = 15, stepY = 8, and each tile spans step + 1 -- the shared seam.
        assert_eq!((c0.w, c0.h), (16.0, 9.0));
        let c1 = m.tiles[1].coarse.expect("tile 1 coarse");
        assert_eq!(c1.x, 19.0);
        assert_eq!(c0.x + c0.w - 1.0, c1.x, "adjacent tiles must share exactly one coarse column");
    }

    #[test]
    fn file_for_overrides_the_default_naming() {
        let m = build_tile_manifest(&TileManifestOpts { cols: 2, rows: 1, ..opts() },
                                    Some(&|r, c| format!("tiles/refined_{r}_{c}.png")));
        assert_eq!(m.tiles[1].file, "tiles/refined_0_1.png");
    }

    #[test]
    fn integral_numbers_render_without_a_decimal_point() {
        assert_eq!(js_num(16.0), "16");
        assert_eq!(js_num(0.0), "0");
        assert_eq!(js_num(-3.0), "-3");
    }

    #[test]
    fn fractional_numbers_keep_their_shortest_round_trip_form() {
        assert_eq!(js_num(0.5), "0.5");
        assert_eq!(js_num(30.0 / 7.0), "4.285714285714286");
    }

    #[test]
    fn non_finite_numbers_render_as_null_like_json_stringify() {
        assert_eq!(js_num(f64::NAN), "null");
        assert_eq!(js_num(f64::INFINITY), "null");
    }

    #[test]
    fn the_compact_form_has_no_whitespace_at_all() {
        let m = build_tile_manifest(&opts(), None);
        let j = manifest_json(&m, None);
        assert!(!j.contains(' '), "{j}");
        assert!(!j.contains('\n'));
        assert!(j.starts_with(r#"{"schema":2,"version":"TESTVER""#));
    }

    #[test]
    fn the_pretty_form_indents_by_the_requested_width() {
        let m = build_tile_manifest(&opts(), None);
        let j = manifest_json(&m, Some(2));
        assert!(j.starts_with("{\n  \"schema\": 2,\n"), "{j}");
        assert!(j.ends_with("\n}"));
    }

    #[test]
    fn a_string_with_a_quote_is_escaped() {
        let m = build_tile_manifest(&TileManifestOpts { version: "a\"b\\c".into(), ..Default::default() }, None);
        assert!(manifest_json(&m, None).contains(r#""version":"a\"b\\c""#));
    }
}

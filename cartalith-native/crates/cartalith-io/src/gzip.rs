//! gzip for the region export's `.bin` tiles — `UNIFIED_TOOL_PLAN.md`
//! milestone E2.
//!
//! Ported from `Cartalith Gen1 v2.10.html`: `gzipBytes` (line 11582) and
//! `gunzipBytes` (11585), both one-liners over the browser's native
//! `CompressionStream`/`DecompressionStream`.
//!
//! # Why here, and why a crate
//!
//! `cartalith-io` owns what a Cartalith file looks like on disk, and a gzipped
//! `tiles/refined_r_c_rg16.bin.gz` is exactly that — it sits beside
//! [`crate::pack_height16`], which produces the bytes being compressed.
//! `flate2` does the compression, per `PROVENANCE.md`'s "take a crate for
//! anything downstream of the pixels" rule, the same call
//! `cartalith-assets` made for `zip` and `image`.
//!
//! # What cannot match the reference, and what can
//!
//! **The gzip bytes themselves cannot, and were never going to.** The
//! reference's stream is the browser's zlib; this is `miniz_oxide`. Two
//! conforming deflate encoders are free to emit different (equally valid)
//! bit streams for the same input, so a byte-for-byte golden here would be
//! testing which zlib build ran, not whether the port is correct. The
//! reference itself already treats this container as interchangeable: on any
//! platform without `CompressionStream` it silently writes the tile
//! uncompressed instead.
//!
//! What *is* the contract, and what the golden test pins, is the round trip in
//! **both directions**: the reference's `gunzipBytes` recovers this port's
//! output byte for byte, and [`gunzip_bytes`] recovers the reference's. That is
//! the only property any consumer of the archive depends on.
//!
//! **The output is still reproducible run to run**, which the archive's own
//! byte-reproducibility (frozen zip timestamps — see
//! `cartalith_assets::archive`) would otherwise be undermined by: gzip's header
//! carries an MTIME field, and this fixes it at `0` rather than letting the
//! wall clock in.
//!
//! # The one behaviour deliberately not ported
//!
//! `gzipBytes` returns `null` when the platform has no `CompressionStream`, and
//! its caller then stores the tile uncompressed and leaves the manifest reading
//! `"compression": "store"`. That is a browser-availability branch with no Rust
//! equivalent — compression is never missing here — so [`gzip_bytes`] is
//! infallible and the caller's fallback is simply unreachable. The *caller's*
//! side of it is still ported: `cartalith_engine::region_export` writes
//! `"store"` when gzip is off, exactly as the reference does when it fails.

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

/// `gzipBytes(u8)` (reference 11582): gzip a byte slice.
///
/// The header's MTIME is pinned to `0` so the same input always produces the
/// same bytes. Infallible — see the module docs on the reference's `null`
/// return.
pub fn gzip_bytes(data: &[u8]) -> Vec<u8> {
    let mut e = GzEncoder::new(Vec::with_capacity(data.len() / 2 + 32), Compression::default());
    e.write_all(data).expect("writing to a Vec cannot fail");
    e.finish().expect("finishing into a Vec cannot fail")
}

/// `gunzipBytes(u8)` (reference 11585): un-gzip a byte slice.
///
/// `Err` on anything that is not a well-formed gzip stream, where the
/// reference answers `null` from its own `catch`.
pub fn gunzip_bytes(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    GzDecoder::new(data).read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(n: usize) -> Vec<u8> {
        (0..n).map(|i| ((i * 37 + (i >> 4) * 11) & 255) as u8).collect()
    }

    #[test]
    fn round_trips_a_kilobyte_of_structured_bytes() {
        let src = sample(1024);
        assert_eq!(gunzip_bytes(&gzip_bytes(&src)).expect("valid gzip"), src);
    }

    #[test]
    fn round_trips_the_empty_slice() {
        assert_eq!(gunzip_bytes(&gzip_bytes(&[])).expect("valid gzip"), Vec::<u8>::new());
    }

    #[test]
    fn writes_a_real_gzip_header() {
        let z = gzip_bytes(&sample(64));
        assert_eq!(&z[..3], &[0x1f, 0x8b, 0x08], "magic + deflate method");
        assert_eq!(&z[4..8], &[0, 0, 0, 0], "MTIME pinned to 0, so exports are reproducible");
    }

    #[test]
    fn the_same_input_gzips_to_the_same_bytes_twice() {
        let src = sample(4096);
        assert_eq!(gzip_bytes(&src), gzip_bytes(&src));
    }

    #[test]
    fn compressible_input_actually_shrinks() {
        // A packed height tile is highly structured; if this ever stopped
        // shrinking, the caller's `.gz` suffix would be a lie.
        let src: Vec<u8> = (0..4096).map(|i| (i % 7) as u8).collect();
        assert!(gzip_bytes(&src).len() < src.len() / 4);
    }

    #[test]
    fn refuses_bytes_that_are_not_gzip() {
        assert!(gunzip_bytes(b"not a gzip stream at all").is_err());
    }
}

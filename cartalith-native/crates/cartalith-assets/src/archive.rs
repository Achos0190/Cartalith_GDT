//! Pack `.zip` read/write — the reference's `unzipAny` (line 12210) and
//! `zipStore` (line 12009) in Rust terms, plus the entry *ordering* its own
//! exporter (`PackManifestBuilder.build()`, line 26964) writes.
//!
//! Behind the `zip` feature so [`crate::manifest`] keeps the property
//! milestone 1 gave it: a manifest model with no archive dependency at all,
//! reachable with `default-features = false`. Everything here is a thin,
//! deliberate layer over the `zip` crate — `PROVENANCE.md`'s "take a crate for
//! anything downstream of the pixels" rule, which covers the container too.
//!
//! ## What is ported and what is delegated
//!
//! The container itself is delegated: a pack is a plain PKZIP, STORE or raw
//! DEFLATE, read via the central directory. `zip::ZipArchive` does exactly
//! that. What is *ported* is the reference's own policy around it, because
//! that policy is visible in the bytes a pack author gets:
//!
//! - **`.png` entries are STORED, never deflated.** A PNG is already
//!   internally DEFLATE-compressed, so re-compressing it is wasted CPU for no
//!   size gain (the reference says so in its own comment at line ~11996).
//!   Everything else — in practice `pack.json` — is deflated.
//! - **Timestamps are frozen at 1980-01-01 00:00:00.** `zipStore` hardcodes
//!   the DOS date word to `0x0021` and the time word to `0`, so two exports of
//!   the same pack are byte-identical. `zip`'s own default is the *current*
//!   time, so this file sets [`zip::DateTime::default`] explicitly rather than
//!   inheriting it.
//! - **`pack.json` is written last**, after every image — the reference's
//!   exporter appends it once the family walk is finished. Entry order is not
//!   semantically load-bearing (the reader keys by name), but it is what a
//!   reference-written pack looks like, and a diff against one should be empty.
//! - **No directory entries.** `zipStore` writes files only; `textures/` and
//!   friends exist purely as name prefixes.
//! - **Names are taken verbatim.** The reference does no path normalisation on
//!   read — no stripping of a wrapping folder, no backslash rewriting — which
//!   is precisely why zipping a *folder* rather than its *contents* produces a
//!   pack whose `pack.json` is at `MyPack/pack.json` and therefore not found.
//!   That failure is real, reported by the reference's own error message
//!   ("try re-zipping the folder…"), and is preserved rather than papered over.
//!
//! One deliberate non-port: `zipStore` also falls back to STORE when the
//! deflated bytes did not come out *smaller* (or when the browser has no
//! `CompressionStream` at all — the `file://`-degrades-gracefully rule). Both
//! are browser-side size/availability concerns; neither changes what any
//! reader sees, and Rust has no "compression might be missing" case. A
//! non-`.png` entry is simply deflated here.
//!
//! Nor is `unzipStore` ported: it is `unzipAny`'s fallback for an archive with
//! no readable central directory, and it answers `null` for every deflated
//! entry — a browser-quirk defence against a truncated `ArrayBuffer`, not a
//! format variant. `zip::ZipArchive` requires the central directory and errors
//! cleanly without it, which is the better answer.

use crate::manifest::{MANIFEST_JSON, PackError, PackManifest};
use crate::slots::Family;
use std::collections::BTreeMap;
use std::io::{Read, Seek, Write};
use zip::CompressionMethod;

/// Entry name → bytes, the shape [`crate::parse_pack_entries`] consumes and
/// the reference's own in-memory `zip` object holds.
pub type PackEntries = BTreeMap<String, Vec<u8>>;

/// What went wrong reading or writing a pack archive.
#[derive(Debug)]
pub enum ArchiveError {
    /// The container itself is unreadable — truncated, no central directory,
    /// not a zip.
    Zip(zip::result::ZipError),
    /// An I/O failure on the underlying reader or writer.
    Io(std::io::Error),
    /// An entry uses a compression method neither the reference nor this port
    /// reads. The reference throws the same sentence.
    UnsupportedMethod {
        /// The raw method number out of the central directory.
        method: u16,
        /// The entry it applies to.
        name: String,
    },
    /// The manifest is missing or malformed ([`PackError`]) — the only content
    /// failure a pack has.
    Pack(PackError),
    /// [`write_pack`] was handed a manifest referencing a file the image map
    /// does not carry. Exporting that would write a pack whose own parser
    /// warns about every missing slot, so it fails loudly instead.
    MissingImage(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::Zip(e) => write!(f, "zip error: {e}"),
            ArchiveError::Io(e) => write!(f, "io error: {e}"),
            // Verbatim the reference's own thrown message (line 12225).
            ArchiveError::UnsupportedMethod { method, name } => {
                write!(f, "unsupported zip method {method} for {name}")
            }
            ArchiveError::Pack(e) => write!(f, "{e}"),
            ArchiveError::MissingImage(p) => write!(f, "pack image missing from export: {p}"),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArchiveError::Zip(e) => Some(e),
            ArchiveError::Io(e) => Some(e),
            ArchiveError::Pack(e) => Some(e),
            ArchiveError::UnsupportedMethod { .. } | ArchiveError::MissingImage(_) => None,
        }
    }
}

impl From<zip::result::ZipError> for ArchiveError {
    fn from(e: zip::result::ZipError) -> Self {
        ArchiveError::Zip(e)
    }
}

impl From<std::io::Error> for ArchiveError {
    fn from(e: std::io::Error) -> Self {
        ArchiveError::Io(e)
    }
}

impl From<PackError> for ArchiveError {
    fn from(e: PackError) -> Self {
        ArchiveError::Pack(e)
    }
}

/// Read every entry of a pack `.zip` into memory, keyed by its name exactly as
/// the central directory spells it (reference `unzipAny`, line 12210).
///
/// Takes anything `Read + Seek` — a `File`, a `Cursor<Vec<u8>>`, an
/// embedded-in-a-save byte slice.
///
/// Faithful to the reference in three ways that matter:
///
/// - **Directory entries are kept**, as zero-byte members. The reference walks
///   the central directory and stores whatever it finds; a pack made by
///   right-click-compress carries them and nothing downstream is confused by
///   them, since no manifest path ever ends in `/`.
/// - **A duplicate name wins by last occurrence**, matching assignment into a
///   JavaScript object.
/// - **An unrecognised compression method is an error**, not a skipped entry —
///   `ArchiveError::UnsupportedMethod`, worded as the reference words it.
pub fn read_pack_entries<R: Read + Seek>(reader: R) -> Result<PackEntries, ArchiveError> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut out = PackEntries::new();
    for i in 0..archive.len() {
        // `by_index_raw` reads the entry's metadata without instantiating a
        // decompressor, so an unsupported method can be reported with its own
        // number and name instead of a generic "unsupported archive".
        let (name, method) = {
            let entry = archive.by_index_raw(i)?;
            (entry.name().to_string(), entry.compression())
        };
        if !matches!(method, CompressionMethod::Stored | CompressionMethod::Deflated) {
            #[allow(deprecated)] // the reference's error string carries the raw number
            let method = method.to_u16();
            return Err(ArchiveError::UnsupportedMethod { method, name });
        }
        let mut entry = archive.by_index(i)?;
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        out.insert(name, buf);
    }
    Ok(out)
}

/// Read a pack `.zip` and validate its manifest in one step — the whole of the
/// reference's `loadAssetPack` minus the image decoding.
///
/// Returns the validated manifest (warnings included; they are data, never
/// errors) alongside the raw entries, because a caller that is about to decode
/// PNGs needs both and reading the archive twice would be silly.
pub fn read_pack<R: Read + Seek>(
    reader: R,
) -> Result<(PackManifest, PackEntries), ArchiveError> {
    let entries = read_pack_entries(reader)?;
    let manifest = crate::parse_pack_entries(&entries)?;
    Ok((manifest, entries))
}

/// `zipStore(entries)` (reference line 12009): write named byte blobs into a
/// PKZIP, in the order given.
///
/// This is the reference's *only* zip writer, and it has three callers there,
/// not one: the asset-pack exporter, the project `.zip` export, and — since
/// `UNIFIED_TOOL_PLAN.md` milestone E2 — the region-tile export
/// (`cartalith_engine::region_export`). So it is spelled once, here, where
/// milestone 2 already established both of its conventions and verified them
/// against a real reference-written archive. [`write_pack_entries`] is the
/// pack-flavoured name for the same function.
///
/// Three behaviours, all observable in the bytes:
///
/// - **`.png` (case-insensitive) is STORED, never deflated.** A PNG is already
///   internally DEFLATE-compressed, so re-compressing it is wasted CPU for no
///   size gain.
/// - **Anything else is deflated only if that actually makes it smaller**,
///   otherwise it falls back to STORE. The reference does this too, and it is
///   not a corner case: a region export's `params.json` is often a few bytes,
///   where the deflate header alone costs more than it saves. Milestone 2 left
///   this un-ported (it read as a browser size concern); milestone E2 measured
///   the reference and found the region export reaches it on its very first
///   entry, so it is ported now. The decision is made by compressing once with
///   `flate2` and comparing lengths — the same encoder `zip` itself uses, so
///   the measurement and the eventual write agree.
/// - **Every timestamp is 1980-01-01 00:00:00**, because `zipStore` hardcodes
///   the DOS date word to `0x0021` and the time word to `0`. `zip`'s own
///   default is the wall clock, which would make two exports of the same data
///   differ.
///
/// Two entries with the same name are written twice, as the reference would;
/// neither its exporters nor this port's can generate that.
pub fn zip_store<W: Write + Seek>(writer: W, entries: &[(&str, &[u8])]) -> Result<(), ArchiveError> {
    let mut zw = zip::ZipWriter::new(writer);
    for (name, data) in entries {
        let method = if name.to_ascii_lowercase().ends_with(".png") || !deflate_helps(data) {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(method)
            // `zip`'s own default here is the current clock; the reference
            // hardcodes the DOS epoch, and a reproducible export is worth
            // more than a real timestamp.
            .last_modified_time(zip::DateTime::default())
            .large_file(false);
        zw.start_file(*name, opts)?;
        zw.write_all(data)?;
    }
    zw.finish()?;
    Ok(())
}

/// [`zip_store`] straight into a `Vec`, which is what every caller in this
/// port actually wants — the reference hands its result to
/// `URL.createObjectURL` as one in-memory `Blob`.
pub fn zip_store_bytes(entries: &[(&str, &[u8])]) -> Result<Vec<u8>, ArchiveError> {
    let mut buf = Vec::new();
    zip_store(std::io::Cursor::new(&mut buf), entries)?;
    Ok(buf)
}

/// `zipStore`'s `cbytes && cbytes.length < data.length` test: would raw
/// DEFLATE actually shrink this entry?
fn deflate_helps(data: &[u8]) -> bool {
    use flate2::{Compression, write::DeflateEncoder};
    let mut e = DeflateEncoder::new(Vec::with_capacity(data.len()), Compression::default());
    if e.write_all(data).is_err() {
        return false;
    }
    match e.finish() {
        Ok(c) => c.len() < data.len(),
        Err(_) => false, // the reference's own `catch` -> null -> store
    }
}

/// Write entries into a pack `.zip`, in the order given.
///
/// The pack-facing name for [`zip_store`], which carries the documentation of
/// what the format actually does. The caller owns the ordering because the
/// reference's exporter does: `pack.json` goes last, after the images.
/// [`write_pack`] applies that ordering for you.
pub fn write_pack_entries<W: Write + Seek>(
    writer: W,
    entries: &[(&str, &[u8])],
) -> Result<(), ArchiveError> {
    zip_store(writer, entries)
}

/// Write a validated manifest and its images back out as a pack `.zip`, the
/// way the reference's own exporter does.
///
/// `images` maps a manifest-declared path to its PNG bytes — exactly the map
/// [`read_pack_entries`] returns, so `read_pack` → `write_pack` is a direct
/// round-trip. Images are written in the exporter's traversal order (the
/// families in [`Family::ALL`] order, each family's frozen slot order within
/// it, variants in manifest order, custom sets last), then `pack.json` from
/// [`PackManifest::to_pack_json`] as the final entry.
///
/// A path the manifest declares but `images` does not carry is
/// [`ArchiveError::MissingImage`]. A path declared by two different slots is
/// written once, at its first occurrence — `zipStore` would happily emit it
/// twice, but only because its own exporter can never generate that case.
pub fn write_pack<W: Write + Seek>(
    writer: W,
    manifest: &PackManifest,
    images: &PackEntries,
) -> Result<(), ArchiveError> {
    let order = export_order(manifest);
    let pack_json = manifest.to_pack_json();
    let mut entries: Vec<(&str, &[u8])> = Vec::with_capacity(order.len() + 1);
    for p in &order {
        let bytes = images
            .get(*p)
            .ok_or_else(|| ArchiveError::MissingImage((*p).to_string()))?;
        entries.push((p, bytes.as_slice()));
    }
    entries.push((MANIFEST_JSON, pack_json.as_bytes()));
    write_pack_entries(writer, &entries)
}

/// The image paths a manifest declares, in the order the reference's exporter
/// writes them: [`Family::ALL`] order, each family's frozen slot order within
/// it, variants in manifest order, and the open custom sets last. Duplicates
/// are dropped at their later occurrences.
fn export_order(manifest: &PackManifest) -> Vec<&str> {
    let mut all: Vec<&str> = Vec::new();
    for fam in Family::ALL {
        match fam {
            Family::Textures | Family::Biomes | Family::Terrains => {
                let m = match fam {
                    Family::Textures => &manifest.textures,
                    Family::Biomes => &manifest.biomes,
                    _ => &manifest.terrains,
                };
                all.extend(fam.slots().iter().filter_map(|s| m.get(s).map(String::as_str)));
            }
            Family::Icons => {
                for slot in fam.slots() {
                    if let Some(v) = manifest.icons.get(slot) {
                        all.extend(v.iter().map(String::as_str));
                    }
                }
            }
            Family::Settlement | Family::Trait | Family::Poi => {
                let m = manifest.structures.family(fam);
                for slot in fam.slots() {
                    if let Some(v) = m.get(slot) {
                        all.extend(v.iter().map(String::as_str));
                    }
                }
            }
            Family::Custom => {
                for (_set, slots) in manifest.custom.iter() {
                    for (_slot, v) in slots.iter() {
                        all.extend(v.iter().map(String::as_str));
                    }
                }
            }
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    all.retain(|p| seen.insert(*p));
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RawManifest, parse_pack_manifest};
    use std::collections::BTreeSet;
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;

    /// Builds a zip by hand so a test can control the method, the names and
    /// whether directory entries are present.
    fn build(files: &[(&str, &[u8], CompressionMethod)], dirs: &[&str]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zw = zip::ZipWriter::new(Cursor::new(&mut buf));
            for d in dirs {
                zw.add_directory(*d, SimpleFileOptions::default()).unwrap();
            }
            for (name, data, method) in files {
                zw.start_file(*name, SimpleFileOptions::default().compression_method(*method))
                    .unwrap();
                zw.write_all(data).unwrap();
            }
            zw.finish().unwrap();
        }
        buf
    }

    const MANIFEST: &[u8] =
        br#"{"schema":1,"name":"T","license":"CC0","textures":{"grass":"textures/grass.png"}}"#;

    #[test]
    fn reads_stored_and_deflated_entries_alike() {
        let zip = build(
            &[
                ("pack.json", MANIFEST, CompressionMethod::Deflated),
                ("textures/grass.png", b"PNG", CompressionMethod::Stored),
            ],
            &[],
        );
        let (manifest, entries) = read_pack(Cursor::new(zip)).unwrap();
        assert_eq!(entries["textures/grass.png"], b"PNG");
        assert_eq!(manifest.name, "T");
        assert!(manifest.warnings.is_empty());
    }

    /// The reference walks the central directory and keeps whatever it finds,
    /// directory records included. Harmless — no manifest path ends in `/` —
    /// and a pack made by right-click-compress really does carry them.
    #[test]
    fn directory_entries_survive_as_empty_members() {
        let zip = build(
            &[
                ("pack.json", MANIFEST, CompressionMethod::Deflated),
                ("textures/grass.png", b"PNG", CompressionMethod::Stored),
            ],
            &["textures/"],
        );
        let entries = read_pack_entries(Cursor::new(zip)).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries["textures/"].is_empty());
        assert!(crate::parse_pack_entries(&entries).unwrap().warnings.is_empty());
    }

    /// Zipping the *folder* rather than its contents is the classic pack
    /// authoring mistake. The reference does no prefix stripping, so the
    /// manifest is simply not found — and that is the behaviour to keep, since
    /// silently guessing a root would make the manifest's own paths ambiguous.
    #[test]
    fn a_wrapping_folder_is_not_stripped() {
        let zip = build(
            &[
                ("MyPack/pack.json", MANIFEST, CompressionMethod::Deflated),
                ("MyPack/textures/grass.png", b"PNG", CompressionMethod::Stored),
            ],
            &[],
        );
        match read_pack(Cursor::new(zip)) {
            Err(ArchiveError::Pack(PackError::NoManifest)) => {}
            other => panic!("expected NoManifest, got {:?}", other.map(|(m, _)| m.name)),
        }
    }

    #[test]
    fn an_unreadable_compression_method_names_itself_the_way_the_reference_does() {
        // Method 93 (Zstandard) — a real method neither side decodes here.
        let mut zip = build(&[("pack.json", MANIFEST, CompressionMethod::Stored)], &[]);
        // Patch the method in both the local header and its central-directory
        // copy, which is the one `unzipAny` and `ZipArchive` actually read.
        zip[8] = 93;
        let cd = zip.windows(4).position(|w| w == [0x50, 0x4b, 0x01, 0x02]).unwrap();
        zip[cd + 10] = 93;
        match read_pack_entries(Cursor::new(zip)) {
            Err(e @ ArchiveError::UnsupportedMethod { .. }) => {
                assert_eq!(e.to_string(), "unsupported zip method 93 for pack.json");
            }
            other => panic!("expected UnsupportedMethod, got {other:?}"),
        }
    }

    #[test]
    fn a_pack_with_no_manifest_fails_with_the_references_own_message() {
        let zip = build(&[("textures/grass.png", b"PNG", CompressionMethod::Stored)], &[]);
        match read_pack(Cursor::new(zip)) {
            Err(e @ ArchiveError::Pack(PackError::NoManifest)) => {
                assert_eq!(e.to_string(), "pack has no pack.json or pack.csv");
            }
            other => panic!("expected NoManifest, got {:?}", other.map(|(m, _)| m.name)),
        }
    }

    /// `pack.csv` is a real second input format, so the archive layer must not
    /// quietly assume `pack.json`.
    #[test]
    fn a_csv_only_pack_opens() {
        let zip = build(
            &[
                (
                    "pack.csv",
                    b"type,slot,file\ntexture,grass,textures/grass.png\n",
                    CompressionMethod::Deflated,
                ),
                ("textures/grass.png", b"PNG", CompressionMethod::Stored),
            ],
            &[],
        );
        let (manifest, _) = read_pack(Cursor::new(zip)).unwrap();
        assert_eq!(
            manifest.textures.get("grass").map(String::as_str),
            Some("textures/grass.png")
        );
    }

    #[test]
    fn png_entries_are_stored_and_everything_else_deflated() {
        let png = vec![b'A'; 4096]; // trivially compressible, yet must stay STORED
        let text = vec![b'B'; 4096];
        let mut buf = Vec::new();
        write_pack_entries(
            Cursor::new(&mut buf),
            &[("icons/x.PNG", &png), ("notes.txt", &text)],
        )
        .unwrap();
        let mut a = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        assert_eq!(a.by_index_raw(0).unwrap().compression(), CompressionMethod::Stored);
        assert_eq!(a.by_index_raw(0).unwrap().compressed_size(), 4096);
        assert_eq!(a.by_index_raw(1).unwrap().compression(), CompressionMethod::Deflated);
        assert!(a.by_index_raw(1).unwrap().compressed_size() < 4096);
    }

    #[test]
    fn writing_the_same_pack_twice_gives_the_same_bytes() {
        let data: Vec<u8> = (0..64u8).collect();
        let write = || {
            let mut buf = Vec::new();
            write_pack_entries(Cursor::new(&mut buf), &[("a.png", &data), ("b.json", &data)])
                .unwrap();
            buf
        };
        assert_eq!(write(), write(), "frozen timestamps make exports reproducible");
    }

    #[test]
    fn write_pack_refuses_to_export_a_manifest_whose_image_is_missing() {
        let raw: RawManifest = serde_json::from_slice(MANIFEST).unwrap();
        let files: BTreeSet<String> = ["textures/grass.png".to_string()].into_iter().collect();
        let manifest = parse_pack_manifest(&raw, &files);
        let mut buf = Vec::new();
        match write_pack(Cursor::new(&mut buf), &manifest, &PackEntries::new()) {
            Err(ArchiveError::MissingImage(p)) => assert_eq!(p, "textures/grass.png"),
            other => panic!("expected MissingImage, got {other:?}"),
        }
    }

    /// Two slots may legitimately point at the same file; the archive carries
    /// it once.
    #[test]
    fn a_shared_path_is_written_once() {
        let raw: RawManifest = serde_json::from_str(
            r#"{"textures":{"grass":"t.png","rock":"t.png"},"icons":{"hill":["t.png"]}}"#,
        )
        .unwrap();
        let files: BTreeSet<String> = ["t.png".to_string()].into_iter().collect();
        let manifest = parse_pack_manifest(&raw, &files);
        let mut images = PackEntries::new();
        images.insert("t.png".to_string(), b"PNG".to_vec());
        let mut buf = Vec::new();
        write_pack(Cursor::new(&mut buf), &manifest, &images).unwrap();
        let entries = read_pack_entries(Cursor::new(buf)).unwrap();
        assert_eq!(entries.len(), 2, "t.png plus pack.json");
    }
}

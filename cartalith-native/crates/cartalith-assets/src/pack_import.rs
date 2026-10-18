//! Opening a pack `.zip` **into the Asset Library** (`ASSET_PACK_TOOLS_PROPOSAL.md`
//! AP-1, Ruling CQ Q1): the reference's `AssetImporter.importPackZip` (v2.11
//! line 27715, "Load an existing pack ZIP (schema 1 or 2) into the library for
//! editing"), split into the two halves that need no pixels.
//!
//! - [`read_pack_entries_limited`] reads the archive under hard caps, because
//!   a pack opened for editing is a file somebody else made. The renderer's
//!   own reader, [`crate::read_pack_entries`], is deliberately untouched: it is
//!   golden-covered and the map's pack path does not go through here.
//! - [`plan_pack_import`] walks the **raw** manifest in the reference's own
//!   section order and says, per slot, which library slot each path lands in
//!   and which paths are refused before any decode is attempted.
//!
//! Decoding and the store writes live in `cartalith-godot`'s
//! `asset_bridge::AssetLibrarySession::import_pack`, because the colour-managed
//! decode (`colour_import::decode_png_managed`, ET-E2) is a dependency of that
//! crate and not of this one.
//!
//! # Why the raw manifest, not [`crate::parse_pack_manifest`]
//!
//! The reference's `importPackZip` reads `m` straight off `JSON.parse` and
//! routes a slot id outside the frozen vocabulary to a custom slot in the
//! `Imported` set with a warning. `parse_pack_manifest` (the renderer's
//! validator) *drops* such a slot. Using it here would lose art the reference
//! keeps, so this module re-walks the raw sections itself.
//!
//! # What must never happen here
//!
//! Nothing in this module may allocate in proportion to a number the archive
//! *claims* (a central-directory size, a PNG's IHDR dimensions) before that
//! number has been checked against [`ImportLimits`]. Every read is bounded by
//! `Read::take` at the cap plus one byte, so a header that lies about its size
//! still stops at the cap.

use crate::archive::PackEntries;
use crate::manifest::{MANIFEST_CSV, MANIFEST_JSON, Paths, RawManifest, parse_pack_csv};
use crate::ordered_map::OrderedMap;
use crate::slots::Family;
use std::io::{Cursor, Read};
use zip::CompressionMethod;

/// The caps an untrusted pack is read under. Every figure is a **labelled
/// judgement** (no source found that fixes one), sized against what a real
/// pack holds: the frozen vocabulary is 79 slots, the reference fixture
/// (`tests/fixtures/reference_pack.zip`) is 18 entries and about 23 KB, and a
/// bake is at most 512² (`Family::size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportLimits {
    /// The archive file itself. Judgement: 256 MiB is far above any pack of
    /// 512² and 256² PNGs, and below what the phone can hold twice (the shell
    /// reads the file into memory before handing it over).
    pub max_archive_bytes: u64,
    /// Entries in the central directory, directories included. Judgement:
    /// 4 096 is fifty times the frozen vocabulary.
    pub max_entries: usize,
    /// One entry, **decompressed**. Judgement: 64 MiB is a 4 096² RGBA image
    /// stored with no compression at all.
    pub max_entry_bytes: u64,
    /// Every entry together, decompressed — the zip-bomb cap. Judgement:
    /// 512 MiB.
    pub max_total_bytes: u64,
    /// `pack.json` / `pack.csv`, decompressed. Judgement: 4 MiB is about a
    /// hundred thousand slot paths.
    pub max_manifest_bytes: u64,
    /// One side of one image, from its IHDR, checked before decoding.
    /// Judgement: 8 192 px.
    pub max_image_side: u32,
    /// One image's pixel count, from its IHDR. Judgement: 4 096² (64 MiB as
    /// RGBA8), the size `ASSET_PACK_TOOLS_PROPOSAL.md` §5 names as the large
    /// case the session already keeps at source size.
    pub max_image_pixels: u64,
}

impl ImportLimits {
    /// The caps the app uses. See each field for its reasoning.
    pub const DEFAULT: ImportLimits = ImportLimits {
        max_archive_bytes: 256 * 1024 * 1024,
        max_entries: 4096,
        max_entry_bytes: 64 * 1024 * 1024,
        max_total_bytes: 512 * 1024 * 1024,
        max_manifest_bytes: 4 * 1024 * 1024,
        max_image_side: 8192,
        max_image_pixels: 4096 * 4096,
    };
}

impl Default for ImportLimits {
    /// [`ImportLimits::DEFAULT`], so `default()` and the named constant can
    /// never disagree.
    fn default() -> Self {
        ImportLimits::DEFAULT
    }
}

/// Why a whole pack was refused. Every variant's `Display` is a sentence a
/// user can act on; none of them is a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportRefusal {
    /// The file is larger than [`ImportLimits::max_archive_bytes`].
    ArchiveTooLarge { bytes: u64, cap: u64 },
    /// The container is unreadable: not a zip, truncated, encrypted.
    NotAZip(String),
    /// More central-directory entries than [`ImportLimits::max_entries`].
    TooManyEntries { count: usize, cap: usize },
    /// An entry name that is absolute, climbs out with `..`, names a drive,
    /// or carries a control character. A pack never needs one, so its
    /// presence is treated as hostile and the whole archive is refused.
    UnsafeName { name: String, why: &'static str },
    /// A compression method other than STORE or DEFLATE.
    UnsupportedMethod { name: String, method: u16 },
    /// One entry decompresses past [`ImportLimits::max_entry_bytes`].
    EntryTooLarge { name: String, cap: u64 },
    /// The whole archive decompresses past [`ImportLimits::max_total_bytes`].
    TotalTooLarge { cap: u64 },
    /// Neither `pack.json` nor `pack.csv` is present (the reference's own
    /// message).
    NoManifest,
    /// The manifest is larger than [`ImportLimits::max_manifest_bytes`].
    ManifestTooLarge { cap: u64 },
    /// `pack.json` is not valid JSON of the manifest's shape.
    BadManifest(String),
}

impl std::fmt::Display for ImportRefusal {
    /// One plain sentence per variant, naming the cap or the entry concerned.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mib = |b: u64| b / (1024 * 1024);
        match self {
            ImportRefusal::ArchiveTooLarge { bytes, cap } => {
                write!(f, "the file is {} MiB, over the {} MiB limit for a pack", mib(*bytes), mib(*cap))
            }
            ImportRefusal::NotAZip(e) => write!(f, "not a readable .zip ({e})"),
            ImportRefusal::TooManyEntries { count, cap } => {
                write!(f, "the archive lists {count} entries, over the {cap} limit")
            }
            ImportRefusal::UnsafeName { name, why } => {
                write!(f, "refused: entry \"{name}\" {why}")
            }
            ImportRefusal::UnsupportedMethod { name, method } => {
                write!(f, "unsupported zip method {method} for {name}")
            }
            ImportRefusal::EntryTooLarge { name, cap } => {
                write!(f, "refused: \"{name}\" unpacks to more than {} MiB", mib(*cap))
            }
            ImportRefusal::TotalTooLarge { cap } => {
                write!(f, "refused: the archive unpacks to more than {} MiB in total", mib(*cap))
            }
            ImportRefusal::NoManifest => write!(f, "pack has no pack.json or pack.csv"),
            ImportRefusal::ManifestTooLarge { cap } => {
                write!(f, "refused: the manifest is larger than {} MiB", mib(*cap))
            }
            ImportRefusal::BadManifest(e) => write!(f, "pack.json is not a valid manifest ({e})"),
        }
    }
}

impl std::error::Error for ImportRefusal {}

/// Why an archive or manifest path is unsafe, or `None` when it is an
/// ordinary relative path. Backslashes count as separators too, because a
/// Windows-made zip can carry them and a later consumer may honour them.
pub fn unsafe_path_reason(name: &str) -> Option<&'static str> {
    if name.chars().any(|c| c.is_control()) {
        return Some("contains a control character");
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return Some("is an absolute path");
    }
    let b = name.as_bytes();
    if b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
        return Some("names a drive");
    }
    if name.split(['/', '\\']).any(|seg| seg == "..") {
        return Some("climbs out of the pack with \"..\"");
    }
    None
}

/// Read every entry of an untrusted pack `.zip` under `limits`. Same output
/// shape as [`crate::read_pack_entries`] (entry name to bytes, directory
/// records kept as empty members, a duplicate name won by its last
/// occurrence), but every way an archive can make the reader allocate without
/// bound is refused with a reason instead.
pub fn read_pack_entries_limited(bytes: &[u8], limits: &ImportLimits) -> Result<PackEntries, ImportRefusal> {
    if bytes.len() as u64 > limits.max_archive_bytes {
        return Err(ImportRefusal::ArchiveTooLarge { bytes: bytes.len() as u64, cap: limits.max_archive_bytes });
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| ImportRefusal::NotAZip(e.to_string()))?;
    if archive.len() > limits.max_entries {
        return Err(ImportRefusal::TooManyEntries { count: archive.len(), cap: limits.max_entries });
    }
    // Names first, so a hostile name refuses the pack before any byte of it
    // is inflated.
    let mut metas = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i).map_err(|e| ImportRefusal::NotAZip(e.to_string()))?;
        let name = entry.name().to_string();
        if let Some(why) = unsafe_path_reason(&name) {
            return Err(ImportRefusal::UnsafeName { name, why });
        }
        let method = entry.compression();
        if !matches!(method, CompressionMethod::Stored | CompressionMethod::Deflated) {
            #[allow(deprecated)] // the reference's own error text carries the raw number
            let method = method.to_u16();
            return Err(ImportRefusal::UnsupportedMethod { name, method });
        }
        // The declared size is a claim; it only gets an early refusal here.
        // The real bound is the `take` below.
        if entry.size() > limits.max_entry_bytes {
            return Err(ImportRefusal::EntryTooLarge { name, cap: limits.max_entry_bytes });
        }
        metas.push(name);
    }
    let mut out = PackEntries::new();
    let mut total: u64 = 0;
    for (i, name) in metas.into_iter().enumerate() {
        let entry = archive.by_index(i).map_err(|e| ImportRefusal::NotAZip(e.to_string()))?;
        // Never `with_capacity(entry.size())`: that size is the archive's
        // claim, and this function exists to not trust one.
        let mut buf = Vec::new();
        entry
            .take(limits.max_entry_bytes + 1)
            .read_to_end(&mut buf)
            .map_err(|e| ImportRefusal::NotAZip(format!("{name}: {e}")))?;
        if buf.len() as u64 > limits.max_entry_bytes {
            return Err(ImportRefusal::EntryTooLarge { name, cap: limits.max_entry_bytes });
        }
        total += buf.len() as u64;
        if total > limits.max_total_bytes {
            return Err(ImportRefusal::TotalTooLarge { cap: limits.max_total_bytes });
        }
        out.insert(name, buf);
    }
    Ok(out)
}

/// The eight-byte PNG signature (ISO/IEC 15948 §5.2).
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Check that `bytes` start like a PNG and that its IHDR dimensions are
/// within `limits`, **without decoding it**. Returns `(width, height)` or a
/// reason. A decode would allocate `w × h × 4` bytes from a header nobody has
/// checked, which is the image equivalent of a zip bomb.
pub fn check_png_header(bytes: &[u8], limits: &ImportLimits) -> Result<(u32, u32), String> {
    if bytes.len() < 8 || bytes[..8] != PNG_SIGNATURE {
        return Err("not a PNG (the file does not start with the PNG signature)".to_string());
    }
    // Signature (8) + IHDR length (4) + "IHDR" (4) + width (4) + height (4).
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return Err("not a readable PNG (no IHDR chunk where the format puts it)".to_string());
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if w == 0 || h == 0 {
        return Err(format!("not a usable PNG ({w}×{h} pixels)"));
    }
    if w > limits.max_image_side || h > limits.max_image_side {
        return Err(format!("{w}×{h} px is over the {} px limit for one side", limits.max_image_side));
    }
    if u64::from(w) * u64::from(h) > limits.max_image_pixels {
        return Err(format!(
            "{w}×{h} px is over the {} megapixel limit",
            limits.max_image_pixels / 1_000_000
        ));
    }
    Ok((w, h))
}

/// Where one manifest slot's art lands in the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotTarget {
    /// A frozen slot of `family`, by its id.
    Frozen { family: Family, id: String },
    /// A custom slot, created if absent (`AssetDB::add_custom_slot(id, set)`).
    Custom { id: String, set: String },
}

/// One path the manifest lists for a slot, and whether it may be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPath {
    /// The archive path exactly as the manifest spells it.
    pub path: String,
    /// `Some(reason)` when this path must not be imported; checked before
    /// any decode.
    pub refusal: Option<String>,
}

/// One manifest slot, resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSlot {
    /// The manifest section the slot was read from (`"textures"`,
    /// `"structures.poi"`, `"custom"`, ...), for the report.
    pub section: String,
    /// The slot id as the manifest spells it.
    pub manifest_id: String,
    /// Where it lands.
    pub target: SlotTarget,
    /// The paths, in manifest order.
    pub paths: Vec<PlannedPath>,
    /// The reference's own warning when an unknown id was rerouted
    /// (`unknown slot "x" → Custom/Imported`), else `None`.
    pub note: Option<String>,
}

/// A whole pack, resolved: its three metadata strings and every slot in the
/// order the reference's `importPackZip` visits them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportPlan {
    /// `name`, when the manifest carries a non-empty one (the reference's
    /// `if(m.name&&...)`); `None` otherwise, never a placeholder.
    pub name: Option<String>,
    /// `author`, under the same rule.
    pub author: Option<String>,
    /// `license`, under the same rule.
    pub license: Option<String>,
    /// Every slot, in visit order.
    pub slots: Vec<PlannedSlot>,
}

/// The set name the reference gives an unknown frozen-section slot
/// (`AssetDB.addCustomSlot(id,'Imported')`, v2.11 line 27741).
pub const IMPORTED_SET: &str = "Imported";

/// Resolve a pack's raw manifest against a library vocabulary.
///
/// `is_frozen(family, id)` answers whether the library has a frozen slot
/// `id` in `family` (the session passes `AssetDB`'s own registry; the
/// library's POI vocabulary is wider than the pack's, `LIBRARY_POI_SLOTS`).
///
/// Order, from the reference: `textures`, `biomes`, `terrains`, `icons`,
/// `structures.settlement`, `structures.trait`, `structures.poi`, then
/// `custom`. **`seamarks` is this port's addition** (owner ruling 2026-09-02;
/// the reference predates it) and is visited after the structures, before
/// `custom`: the exporter writes it, so leaving it out would make an exported
/// pack lose its sea marks on the way back in.
pub fn plan_pack_import(
    entries: &PackEntries,
    limits: &ImportLimits,
    is_frozen: impl Fn(Family, &str) -> bool,
) -> Result<ImportPlan, ImportRefusal> {
    let raw: RawManifest = if let Some(bytes) = entries.get(MANIFEST_JSON) {
        if bytes.len() as u64 > limits.max_manifest_bytes {
            return Err(ImportRefusal::ManifestTooLarge { cap: limits.max_manifest_bytes });
        }
        serde_json::from_str(&String::from_utf8_lossy(bytes)).map_err(|e| ImportRefusal::BadManifest(e.to_string()))?
    } else if let Some(bytes) = entries.get(MANIFEST_CSV) {
        if bytes.len() as u64 > limits.max_manifest_bytes {
            return Err(ImportRefusal::ManifestTooLarge { cap: limits.max_manifest_bytes });
        }
        parse_pack_csv(&String::from_utf8_lossy(bytes))
    } else {
        return Err(ImportRefusal::NoManifest);
    };
    let nonblank = |s: &Option<String>| s.as_ref().filter(|v| !v.is_empty()).cloned();
    let mut plan = ImportPlan {
        name: nonblank(&raw.name),
        author: nonblank(&raw.author),
        license: nonblank(&raw.license),
        slots: Vec::new(),
    };

    let path_plan = |p: &str| -> PlannedPath {
        let refusal = if let Some(why) = unsafe_path_reason(p) {
            Some(format!("refused: the path {why}"))
        } else if p.is_empty() || !entries.contains_key(p) {
            // The reference's own wording: `warns.push('missing file: '+path)`.
            Some(format!("missing file: {p}"))
        } else {
            None
        };
        PlannedPath { path: p.to_string(), refusal }
    };
    let resolve = |family: Family, id: &str| -> (SlotTarget, Option<String>) {
        if is_frozen(family, id) {
            (SlotTarget::Frozen { family, id: id.to_string() }, None)
        } else {
            (
                SlotTarget::Custom { id: id.to_string(), set: IMPORTED_SET.to_string() },
                // Verbatim the reference's warning, `→` included.
                Some(format!("unknown slot \"{id}\" → Custom/Imported")),
            )
        }
    };

    // Single-image sections: `Option<String>` per slot.
    let singles: [(Family, &OrderedMap<Option<String>>); 3] =
        [(Family::Textures, &raw.textures), (Family::Biomes, &raw.biomes), (Family::Terrains, &raw.terrains)];
    for (family, section) in singles {
        for (id, v) in section.iter() {
            // `null` in the manifest: the reference's `[v]` would push one
            // `undefined` path and warn "missing file"; a slot with no path
            // has nothing to import, so it is skipped silently here.
            let Some(p) = v else { continue };
            let (target, note) = resolve(family, id);
            plan.slots.push(PlannedSlot {
                section: family.key().to_string(),
                manifest_id: id.to_string(),
                target,
                paths: vec![path_plan(p)],
                note,
            });
        }
    }
    let multis: [(Family, &str, &OrderedMap<Option<Paths>>); 5] = [
        (Family::Icons, "icons", &raw.icons),
        (Family::Settlement, "structures.settlement", &raw.structures.settlement),
        (Family::Trait, "structures.trait", &raw.structures.traits),
        (Family::Poi, "structures.poi", &raw.structures.poi),
        (Family::SeaMark, "seamarks", &raw.seamarks),
    ];
    for (family, label, section) in multis {
        for (id, v) in section.iter() {
            let Some(paths) = v else { continue };
            let (target, note) = resolve(family, id);
            plan.slots.push(PlannedSlot {
                section: label.to_string(),
                manifest_id: id.to_string(),
                target,
                paths: paths.as_slice().iter().map(|p| path_plan(p)).collect(),
                note,
            });
        }
    }
    for (set, slots) in raw.custom.iter() {
        for (id, v) in slots.iter() {
            let Some(paths) = v else { continue };
            plan.slots.push(PlannedSlot {
                section: "custom".to_string(),
                manifest_id: id.to_string(),
                target: SlotTarget::Custom { id: id.to_string(), set: set.to_string() },
                paths: paths.as_slice().iter().map(|p| path_plan(p)).collect(),
                note: None,
            });
        }
    }
    Ok(plan)
}

/// The limits, the hostile-archive refusals, the PNG header check and the
/// plan's routing, each on a zip built for the test.
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    /// A zip of `files`, each written with `method`.
    fn zip_of(files: &[(&str, &[u8])], method: CompressionMethod) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zw = zip::ZipWriter::new(Cursor::new(&mut buf));
            for (name, data) in files {
                zw.start_file(*name, SimpleFileOptions::default().compression_method(method)).unwrap();
                zw.write_all(data).unwrap();
            }
            zw.finish().unwrap();
        }
        buf
    }

    /// The first 24 bytes of a PNG of `w` by `h`: enough for the header
    /// check, which must never need more.
    fn png_head(w: u32, h: u32) -> Vec<u8> {
        let mut v = PNG_SIGNATURE.to_vec();
        v.extend_from_slice(&13u32.to_be_bytes());
        v.extend_from_slice(b"IHDR");
        v.extend_from_slice(&w.to_be_bytes());
        v.extend_from_slice(&h.to_be_bytes());
        v
    }

    /// Protects: the default caps are the literal figures the doc comments
    /// justify, so a silent edit to one fails here rather than in a pack.
    #[test]
    fn default_limits_are_the_documented_figures() {
        let l = ImportLimits::default();
        assert_eq!(l.max_archive_bytes, 268_435_456);
        assert_eq!(l.max_entries, 4096);
        assert_eq!(l.max_entry_bytes, 67_108_864);
        assert_eq!(l.max_total_bytes, 536_870_912);
        assert_eq!(l.max_manifest_bytes, 4_194_304);
        assert_eq!(l.max_image_side, 8192);
        assert_eq!(l.max_image_pixels, 16_777_216);
    }

    /// Protects: an ordinary pack reads to the same map the renderer's reader
    /// gives, so the limited reader is not a second format.
    #[test]
    fn an_ordinary_pack_reads_like_the_renderers_reader() {
        let z = zip_of(&[("pack.json", b"{}"), ("icons/hill_01.png", b"x")], CompressionMethod::Deflated);
        let mine = read_pack_entries_limited(&z, &ImportLimits::DEFAULT).unwrap();
        let theirs = crate::read_pack_entries(Cursor::new(&z)).unwrap();
        assert_eq!(mine, theirs);
    }

    /// Protects: a deflate bomb (32 MiB of zeros in a few KB) is refused at
    /// the entry cap with the entry named, and the total cap fires for many
    /// entries that are each under the entry cap.
    #[test]
    fn a_deflate_bomb_is_refused_at_the_cap() {
        let zeros = vec![0u8; 32 * 1024 * 1024];
        let z = zip_of(&[("pack.json", &zeros)], CompressionMethod::Deflated);
        assert!(z.len() < 200_000, "the fixture really is a bomb: {} bytes", z.len());
        let small = ImportLimits { max_entry_bytes: 1024 * 1024, ..ImportLimits::DEFAULT };
        match read_pack_entries_limited(&z, &small) {
            Err(ImportRefusal::EntryTooLarge { name, .. }) => assert_eq!(name, "pack.json"),
            other => panic!("expected EntryTooLarge, got {other:?}"),
        }
        let many: Vec<(String, Vec<u8>)> = (0..8).map(|i| (format!("f{i}.bin"), vec![0u8; 1024 * 1024])).collect();
        let refs: Vec<(&str, &[u8])> = many.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let z = zip_of(&refs, CompressionMethod::Deflated);
        let caps = ImportLimits { max_total_bytes: 4 * 1024 * 1024, ..ImportLimits::DEFAULT };
        assert_eq!(read_pack_entries_limited(&z, &caps), Err(ImportRefusal::TotalTooLarge { cap: 4 * 1024 * 1024 }));
    }

    /// Protects: an entry whose central directory LIES about its size (says
    /// 4 bytes, holds 2 MiB) is still stopped at the cap by the bounded read,
    /// not trusted.
    #[test]
    fn a_lying_size_header_is_still_bounded() {
        let data = vec![7u8; 2 * 1024 * 1024];
        let mut z = zip_of(&[("a.bin", &data)], CompressionMethod::Stored);
        // Patch the central directory's uncompressed size (offset 24 in the
        // CD record) and the local header's (offset 22) to 4.
        let cd = z.windows(4).position(|w| w == [0x50, 0x4b, 0x01, 0x02]).unwrap();
        z[cd + 24..cd + 28].copy_from_slice(&4u32.to_le_bytes());
        z[22..26].copy_from_slice(&4u32.to_le_bytes());
        let caps = ImportLimits { max_entry_bytes: 1024 * 1024, ..ImportLimits::DEFAULT };
        let got = read_pack_entries_limited(&z, &caps);
        // Either the bounded read stops it at the cap, or the zip crate
        // itself refuses the inconsistent entry; both are refusals, and
        // neither may return the 2 MiB.
        assert!(got.is_err(), "a lying header must not yield its bytes: {:?}", got.map(|e| e.len()));
    }

    /// Protects: every path-traversal shape refuses the WHOLE pack before
    /// anything is inflated, and an ordinary nested path does not.
    #[test]
    fn path_traversal_names_refuse_the_pack() {
        for bad in ["../evil.png", "icons/../../evil.png", "/etc/passwd", "\\win.ini", "C:/x.png", "a\\..\\b.png"] {
            let z = zip_of(&[("pack.json", b"{}"), (bad, b"x")], CompressionMethod::Stored);
            match read_pack_entries_limited(&z, &ImportLimits::DEFAULT) {
                Err(ImportRefusal::UnsafeName { name, .. }) => assert_eq!(name, bad),
                other => panic!("{bad}: expected UnsafeName, got {other:?}"),
            }
        }
        assert_eq!(unsafe_path_reason("custom/naval/lighthouse_01.png"), None);
        assert_eq!(unsafe_path_reason("a..b.png"), None, "two dots inside a name are not a climb");
    }

    /// Protects: the entry-count and archive-size caps fire, with the counts
    /// in the refusal.
    #[test]
    fn entry_count_and_archive_size_caps_fire() {
        let files: Vec<(String, Vec<u8>)> = (0..5).map(|i| (format!("f{i}"), vec![1u8])).collect();
        let refs: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let z = zip_of(&refs, CompressionMethod::Stored);
        let caps = ImportLimits { max_entries: 4, ..ImportLimits::DEFAULT };
        assert_eq!(read_pack_entries_limited(&z, &caps), Err(ImportRefusal::TooManyEntries { count: 5, cap: 4 }));
        let caps = ImportLimits { max_archive_bytes: 10, ..ImportLimits::DEFAULT };
        assert!(matches!(read_pack_entries_limited(&z, &caps), Err(ImportRefusal::ArchiveTooLarge { .. })));
        assert!(matches!(read_pack_entries_limited(b"not a zip", &ImportLimits::DEFAULT), Err(ImportRefusal::NotAZip(_))));
    }

    /// Protects: the PNG header check refuses a non-PNG, a zero size, and both
    /// oversize shapes (one long side; a square over the pixel cap), and
    /// passes the 256² sprite case, all from 24 bytes and no decode.
    #[test]
    fn the_png_header_check_reads_only_the_header() {
        let l = ImportLimits::DEFAULT;
        assert_eq!(check_png_header(&png_head(256, 256), &l), Ok((256, 256)));
        assert_eq!(check_png_header(&png_head(4096, 4096), &l), Ok((4096, 4096)));
        assert!(check_png_header(b"GIF89a.................", &l).unwrap_err().contains("not a PNG"));
        assert!(check_png_header(&png_head(0, 5), &l).is_err());
        assert!(check_png_header(&png_head(8193, 1), &l).unwrap_err().contains("8192"));
        assert!(check_png_header(&png_head(4097, 4096), &l).unwrap_err().contains("megapixel"));
    }

    /// Protects: the plan visits the reference's section order, routes an
    /// unknown frozen-section id to Custom/Imported with the reference's own
    /// warning, keeps custom sets as written, and refuses a missing or
    /// traversing path per path rather than per pack.
    #[test]
    fn the_plan_follows_the_references_order_and_routing() {
        let manifest = br#"{"name":"P","license":"","custom":{"Naval":{"lighthouse":["c/l.png"]}},
            "icons":{"mountain":["i/m.png","i/gone.png"],"volcano_x":"i/v.png"},
            "seamarks":{"buoy":["s/b.png"]},
            "textures":{"grass":"t/g.png","rock":"../t/r.png"}}"#;
        let mut entries = PackEntries::new();
        entries.insert("pack.json".into(), manifest.to_vec());
        for p in ["c/l.png", "i/m.png", "i/v.png", "s/b.png", "t/g.png"] {
            entries.insert(p.into(), b"x".to_vec());
        }
        let frozen = |f: Family, id: &str| f.has_slot(id);
        let plan = plan_pack_import(&entries, &ImportLimits::DEFAULT, frozen).unwrap();
        assert_eq!(plan.name.as_deref(), Some("P"));
        assert_eq!(plan.license, None, "an empty licence is absent, never a placeholder");
        let order: Vec<(&str, &str)> = plan.slots.iter().map(|s| (s.section.as_str(), s.manifest_id.as_str())).collect();
        assert_eq!(
            order,
            vec![
                ("textures", "grass"),
                ("textures", "rock"),
                ("icons", "mountain"),
                ("icons", "volcano_x"),
                ("seamarks", "buoy"),
                ("custom", "lighthouse"),
            ]
        );
        assert_eq!(plan.slots[1].paths[0].refusal.as_deref(), Some("refused: the path climbs out of the pack with \"..\""));
        assert_eq!(plan.slots[2].paths[0].refusal, None);
        assert_eq!(plan.slots[2].paths[1].refusal.as_deref(), Some("missing file: i/gone.png"));
        assert_eq!(plan.slots[3].target, SlotTarget::Custom { id: "volcano_x".into(), set: "Imported".into() });
        assert_eq!(plan.slots[3].note.as_deref(), Some("unknown slot \"volcano_x\" → Custom/Imported"));
        assert_eq!(plan.slots[5].target, SlotTarget::Custom { id: "lighthouse".into(), set: "Naval".into() });
    }

    /// Protects: a pack with no manifest and one with a broken manifest are
    /// refused with reasons; a pack.csv-only pack plans from the CSV.
    #[test]
    fn manifest_refusals_and_the_csv_fallback() {
        let frozen = |f: Family, id: &str| f.has_slot(id);
        let mut e = PackEntries::new();
        e.insert("x.png".into(), vec![]);
        assert_eq!(plan_pack_import(&e, &ImportLimits::DEFAULT, frozen), Err(ImportRefusal::NoManifest));
        e.insert("pack.json".into(), b"{ not json".to_vec());
        assert!(matches!(plan_pack_import(&e, &ImportLimits::DEFAULT, frozen), Err(ImportRefusal::BadManifest(_))));
        let mut c = PackEntries::new();
        c.insert("pack.csv".into(), b"type,slot,file\ntexture,grass,g.png\n".to_vec());
        c.insert("g.png".into(), vec![1]);
        let plan = plan_pack_import(&c, &ImportLimits::DEFAULT, frozen).unwrap();
        assert_eq!(plan.slots.len(), 1);
        assert_eq!(plan.slots[0].target, SlotTarget::Frozen { family: Family::Textures, id: "grass".into() });
        let big = ImportLimits { max_manifest_bytes: 4, ..ImportLimits::DEFAULT };
        assert!(matches!(plan_pack_import(&c, &big, frozen), Err(ImportRefusal::ManifestTooLarge { .. })));
    }
}

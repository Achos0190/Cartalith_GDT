//! The channel atlas — `packRGB8`/`unpackRGB8`/`channelAtlasGroups`/
//! `channelAtlasManifest`/`channelAtlasEntries` (reference HTML lines 12333,
//! 12341, 12364, 12387, 12408), the `chanAtlasChk` export option.
//!
//! # What it is
//!
//! The world's **affordance fields** — soil fertility, water access, carrying
//! capacity, settlement suitability, the fifteen resource potentials, and the
//! three categorical rasters (biome, lithology, Köppen) — packed three to an
//! RGB8 PNG, plus one JSON manifest saying which channel of which file holds
//! which field.
//!
//! The reference's own justification, worth keeping because it explains why
//! this is not merely a smaller copy of the `.f32` blobs: *"Smaller,
//! viewable, and directly GPU-samplable by the merged tool later; the
//! full-precision .f32 blobs stay as the master."* A packed atlas is
//! something you can open in an image viewer and something a shader can bind
//! in one texture unit. It is deliberately lossy, and the manifest says so.
//!
//! # Two channel kinds, and why alpha is pinned
//!
//! `unit` channels are `[0,1]` scalars stored as `round(v*255)`; `index`
//! channels are categorical rasters already in `0..255` and are stored raw.
//! [`ChannelKind`] is that distinction, and it is carried into the manifest
//! so a reader knows whether to divide by 255.
//!
//! Alpha is `255` on every pixel, always. The reference is explicit that this
//! avoids *"the canvas premultiplied-alpha round-trip corruption that varying
//! alpha would cause"* — a browser concern this port does not literally
//! share, since it encodes through the `image` crate rather than a canvas.
//! It is reproduced anyway: the whole point of the format is that a file
//! written here is readable by the reference and vice versa, and a channel
//! that means "soil fertility" in one and "opacity" in the other would break
//! that for no gain.
//!
//! # This is not the export raster
//!
//! `map.png` (the `bakeRes`/`bakeTiles` controls) is a *picture* of the
//! world, rendered through the whole material path — `cartalith-godot`'s
//! `render::bake_rect`. The channel atlas is *data*, one byte per field per
//! cell, at grid resolution and nothing else. They ride in the same export
//! and share nothing else.

use cartalith_assets::raster::{ImageError, encode_png_rgb8};
use cartalith_jsmath::js_round;

/// `_chanEnc`/`_chanDec`'s two kinds (reference line 12328).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    /// A `[0,1]` scalar, stored as `round(clamp01(v) * 255)`.
    Unit,
    /// A categorical raster already in `0..255`, stored raw and clamped.
    Index,
}

impl ChannelKind {
    fn as_str(self) -> &'static str {
        match self {
            ChannelKind::Unit => "unit",
            ChannelKind::Index => "index",
        }
    }
}

/// One channel's pixels. The two variants are the two kinds' natural storage
/// — `unit` fields are `f32` rasters throughout this workspace and `index`
/// fields are `u8` ones — so a caller cannot accidentally hand a categorical
/// raster to the scalar encoder.
#[derive(Debug, Clone, Copy)]
pub enum ChannelSrc<'a> {
    Unit(&'a [f32]),
    Index(&'a [u8]),
}

impl ChannelSrc<'_> {
    fn kind(&self) -> ChannelKind {
        match self {
            ChannelSrc::Unit(_) => ChannelKind::Unit,
            ChannelSrc::Index(_) => ChannelKind::Index,
        }
    }

    /// `_chanEnc(v, kind)` (reference line 12328).
    ///
    /// `js_round` rather than `f64::round` on the `unit` path: the two
    /// disagree on an exact `.5`, and `v*255` lands on one for every
    /// hundredth-part value a normalized field routinely holds
    /// (`0.5 → 127.5`, `0.1 → 25.5`, `0.3 → 76.5`). Getting that wrong is a
    /// whole byte level on a channel that only has 256 of them.
    fn encode(&self, i: usize) -> u8 {
        match self {
            ChannelSrc::Unit(a) => {
                let v = a.get(i).copied().unwrap_or(0.0) as f64;
                // The reference's own clamp form (`v<0?0:v>1?1:v`), which
                // maps NaN to 255 -- both comparisons are false in JS, so
                // the value falls through to `Math.round(NaN*255)`, i.e.
                // NaN, and `Uint8Array` assignment of NaN stores 0. Rust's
                // `as u8` saturates NaN to 0 as well, so the two agree
                // without a special case; stated because it is exactly the
                // kind of edge `cartalith-rust-conventions` warns about.
                let v = if v < 0.0 {
                    0.0
                } else if v > 1.0 {
                    1.0
                } else {
                    v
                };
                js_round(v * 255.0) as u8
            }
            ChannelSrc::Index(a) => a.get(i).copied().unwrap_or(0),
        }
    }
}

/// One documented channel of one atlas file.
#[derive(Debug, Clone)]
pub struct Channel<'a> {
    /// `"r"`, `"g"` or `"b"`.
    pub ch: &'static str,
    /// The stable machine key (`"soil_fertility"`, `"biome"`, …).
    pub key: String,
    /// The human label the manifest carries (`"Soil fertility"`).
    pub name: String,
    /// `None` leaves the channel at zero and still documents it — the
    /// reference's own `c && c.src ? … : null`, which is how a world with no
    /// Köppen field still produces a well-formed `classes.png`.
    pub src: Option<ChannelSrc<'a>>,
    /// A per-channel index legend, for categorical channels.
    pub manifest: Option<&'static str>,
}

/// One atlas file: up to three channels packed into one RGB8 PNG.
#[derive(Debug, Clone)]
pub struct ChannelGroup<'a> {
    /// The path inside the export (`"atlas/habitat.png"`).
    pub file: String,
    pub channels: Vec<Channel<'a>>,
}

/// `packRGB8(specs, n)` (reference line 12333) — pack up to three fields into
/// one **RGB8** buffer.
///
/// The reference packs RGBA with `alpha=255` because a canvas has no RGB8
/// surface; this returns three bytes per pixel and the encoder writes a
/// PNG with no alpha channel at all. Same decoded values for every channel
/// the manifest documents, one byte per pixel less on disk, and no
/// premultiplication for an alpha that was constant anyway — see the module
/// docs.
pub fn pack_rgb8(specs: &[Option<ChannelSrc<'_>>; 3], n: usize) -> Vec<u8> {
    let mut out = vec![0u8; n * 3];
    for (c, spec) in specs.iter().enumerate() {
        let Some(src) = spec else { continue };
        for i in 0..n {
            out[i * 3 + c] = src.encode(i);
        }
    }
    out
}

/// `unpackRGB8(rgba, n, kinds)` (reference line 12341), against this port's
/// RGB8 layout — the inverse of [`pack_rgb8`], for a reader and for the
/// round-trip test that proves the encoding is lossless where it claims to
/// be.
pub fn unpack_rgb8(rgb: &[u8], n: usize, kinds: [ChannelKind; 3]) -> [Vec<f64>; 3] {
    let mut out = [vec![0f64; n], vec![0f64; n], vec![0f64; n]];
    for (c, kind) in kinds.iter().enumerate() {
        for i in 0..n {
            let b = rgb.get(i * 3 + c).copied().unwrap_or(0) as f64;
            out[c][i] = match kind {
                ChannelKind::Index => b,
                ChannelKind::Unit => b / 255.0,
            };
        }
    }
    out
}

/// `_resourceAtlasGroups(rp)` (reference line 12354) — the resource fields in
/// threes, `atlas/resources_a.png`, `_b`, `_c`, … in key order.
///
/// **Generated from the key list, never hand-listed.** The reference's own
/// v1.31 note says why: the literal it replaced covered the original six
/// keys, and growing the vocabulary to fifteen *"would have silently dropped
/// nine fields out of the channel atlas (and its manifest) with nothing to
/// catch it"*. `keys`/`names` are the caller's `RESOURCE_KEYS`/
/// `RESOURCE_NAMES`, so the file count follows the vocabulary here too.
pub fn resource_groups<'a>(keys: &[&str], names: &[&str], lookup: impl Fn(&str) -> Option<&'a [f32]>) -> Vec<ChannelGroup<'a>> {
    const CH: [&str; 3] = ["r", "g", "b"];
    let mut out = Vec::new();
    for (gi, slice) in keys.chunks(3).enumerate() {
        // `String.fromCharCode(97 + i/3)` — 'a', 'b', 'c', …
        let suffix = (b'a' + gi as u8) as char;
        let channels = slice
            .iter()
            .enumerate()
            .map(|(j, k)| Channel {
                ch: CH[j],
                key: (*k).to_string(),
                name: format!("{} potential", names.get(keys.iter().position(|x| x == k).unwrap_or(j)).copied().unwrap_or(k)),
                src: lookup(k).map(ChannelSrc::Unit),
                manifest: None,
            })
            .collect();
        out.push(ChannelGroup { file: format!("atlas/resources_{suffix}.png"), channels });
    }
    out
}

/// `channelAtlasManifest(groups)` (reference line 12387) — the decode
/// manifest, pretty-printed with the reference's own two-space indent
/// (`JSON.stringify(…, null, 2)`).
///
/// Built with `serde_json::Value` rather than a `Serialize` struct because
/// the shape is a *map keyed by channel letter* whose members vary per file;
/// a struct would need three `Option` fields and would emit them in a fixed
/// order regardless of which channels a group actually documents.
pub fn manifest_json(groups: &[ChannelGroup<'_>], gw: usize, gh: usize, version: &str) -> String {
    let files: Vec<serde_json::Value> = groups
        .iter()
        .map(|grp| {
            let mut ch = serde_json::Map::new();
            for c in &grp.channels {
                let kind = c.src.map(|s| s.kind()).unwrap_or(ChannelKind::Unit);
                ch.insert(
                    c.ch.to_string(),
                    serde_json::json!({
                        "key": c.key,
                        "name": c.name,
                        "kind": kind.as_str(),
                        "range": if kind == ChannelKind::Unit { serde_json::json!([0, 1]) } else { serde_json::json!("categorical") },
                        "manifest": c.manifest,
                    }),
                );
            }
            serde_json::json!({ "file": grp.file, "width": gw, "height": gh, "channels": ch })
        })
        .collect();
    let doc = serde_json::json!({
        "schema": 1,
        "kind": "cartalith-channel-atlas",
        "version": version,
        "encoding": "rgb8",
        "note": "8-bit per channel; row-major GW\u{00d7}GH. unit channels are value\u{00b7}255; index channels are raw categorical indices (see per-channel manifest). The full-precision .f32 / _raster.bin blobs remain the master copies.",
        "width": gw,
        "height": gh,
        "files": files,
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

/// One file in the atlas export, named as it appears inside the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtlasEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// `channelAtlasEntries()` (reference line 12408) — every group as a PNG,
/// then `atlas/index.json`.
///
/// A group with no populated channel at all is **skipped**, not written as a
/// black image: the reference drops a group whose PNG encode returns `null`,
/// and a file of zeros documented as "soil fertility" is worse than an absent
/// one. Skipped groups are dropped from the manifest too, so the two can
/// never disagree about what shipped.
pub fn entries(groups: &[ChannelGroup<'_>], gw: usize, gh: usize, version: &str) -> Result<Vec<AtlasEntry>, ImageError> {
    let n = gw * gh;
    let mut out = Vec::new();
    let mut kept: Vec<ChannelGroup<'_>> = Vec::new();
    for grp in groups {
        if n == 0 || grp.channels.iter().all(|c| c.src.is_none()) {
            continue;
        }
        let mut specs: [Option<ChannelSrc<'_>>; 3] = [None, None, None];
        for c in &grp.channels {
            let slot = match c.ch {
                "r" => 0,
                "g" => 1,
                "b" => 2,
                _ => continue,
            };
            specs[slot] = c.src;
        }
        let png = encode_png_rgb8(gw as u32, gh as u32, pack_rgb8(&specs, n))?;
        out.push(AtlasEntry { name: grp.file.clone(), data: png });
        kept.push(grp.clone());
    }
    if !out.is_empty() {
        out.push(AtlasEntry { name: "atlas/index.json".to_string(), data: manifest_json(&kept, gw, gh, version).into_bytes() });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `_chanEnc`, value by value, against the reference's own definition
    /// read off line 12328 — not against a dump, because the function is
    /// three comparisons and a multiply and a dump would only re-state them.
    ///
    /// The half-way values are the point: `Math.round` and `f64::round`
    /// agree here only because the input is non-negative, and the test pins
    /// the answers so a later switch to a signed field cannot pass silently.
    #[test]
    fn chan_enc_matches_the_reference_definition() {
        let unit = [-1.0f32, 0.0, 0.1, 0.3, 0.5, 0.99609375, 1.0, 2.0];
        let want = [0u8, 0, 26, 77, 128, 254, 255, 255];
        let src = ChannelSrc::Unit(&unit);
        for (i, w) in want.iter().enumerate() {
            assert_eq!(src.encode(i), *w, "unit[{i}] = {}", unit[i]);
        }
        // `0.1*255 = 25.5` and `0.3*255 = 76.5` are exact halves: JS rounds
        // them up, and so must this.
        assert_eq!(src.encode(2), 26, "0.1 must round up, not to even");
        assert_eq!(src.encode(3), 77, "0.3 must round up, not to even");

        let idx = [0u8, 1, 9, 255];
        let src = ChannelSrc::Index(&idx);
        for (i, w) in idx.iter().enumerate() {
            assert_eq!(src.encode(i), *w, "index[{i}] passes through raw");
        }
    }

    #[test]
    fn nan_encodes_to_zero_not_to_a_wrapped_byte() {
        let unit = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY];
        let src = ChannelSrc::Unit(&unit);
        assert_eq!(src.encode(0), 0, "NaN");
        assert_eq!(src.encode(1), 255, "+inf clamps to 1.0");
        assert_eq!(src.encode(2), 0, "-inf clamps to 0.0");
    }

    #[test]
    fn pack_then_unpack_round_trips_within_one_byte_level() {
        let a: Vec<f32> = (0..64).map(|i| i as f32 / 63.0).collect();
        let b: Vec<u8> = (0..64).map(|i| (i % 7) as u8).collect();
        let specs = [Some(ChannelSrc::Unit(&a)), Some(ChannelSrc::Index(&b)), None];
        let packed = pack_rgb8(&specs, 64);
        assert_eq!(packed.len(), 64 * 3);
        let got = unpack_rgb8(&packed, 64, [ChannelKind::Unit, ChannelKind::Index, ChannelKind::Unit]);
        for i in 0..64 {
            assert!((got[0][i] - a[i] as f64).abs() <= 0.5 / 255.0, "unit channel lost more than half a level at {i}");
            assert_eq!(got[1][i], b[i] as f64, "index channel must be exact at {i}");
            assert_eq!(got[2][i], 0.0, "an absent channel decodes as zero");
        }
    }

    /// The v1.31 rule, as a test: the number of resource files follows the
    /// key vocabulary. Fifteen keys is five files, not the two the pre-v1.31
    /// literal had.
    #[test]
    fn resource_groups_follow_the_key_vocabulary() {
        let keys: Vec<&str> = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o"];
        let names: Vec<&str> = keys.clone();
        let field = vec![0.5f32; 4];
        let groups = resource_groups(&keys, &names, |_| Some(&field));
        assert_eq!(groups.len(), 5, "15 keys must produce ceil(15/3) files");
        assert_eq!(groups[0].file, "atlas/resources_a.png");
        assert_eq!(groups[4].file, "atlas/resources_e.png");
        assert_eq!(groups[4].channels.len(), 3);
        assert_eq!(groups[0].channels[0].ch, "r");
        assert_eq!(groups[0].channels[2].ch, "b");
        assert_eq!(groups[0].channels[1].name, "b potential");
        // A short final group keeps only the channels it has.
        let short = resource_groups(&keys[..4], &names[..4], |_| Some(&field));
        assert_eq!(short.len(), 2);
        assert_eq!(short[1].channels.len(), 1);
    }

    #[test]
    fn entries_writes_real_pngs_and_a_matching_manifest() {
        let (gw, gh) = (8usize, 5usize);
        let n = gw * gh;
        let soil: Vec<f32> = (0..n).map(|i| i as f32 / n as f32).collect();
        let biome: Vec<u8> = (0..n).map(|i| (i % 11) as u8).collect();
        let groups = vec![
            ChannelGroup {
                file: "atlas/habitat.png".into(),
                channels: vec![Channel { ch: "r", key: "soil_fertility".into(), name: "Soil fertility".into(), src: Some(ChannelSrc::Unit(&soil)), manifest: None }],
            },
            ChannelGroup {
                file: "atlas/classes.png".into(),
                channels: vec![Channel { ch: "r", key: "biome".into(), name: "Biome index".into(), src: Some(ChannelSrc::Index(&biome)), manifest: Some("biome_index.json") }],
            },
            // Nothing populated -- must be dropped, not written black.
            ChannelGroup {
                file: "atlas/empty.png".into(),
                channels: vec![Channel { ch: "r", key: "koppen".into(), name: "K\u{f6}ppen index".into(), src: None, manifest: None }],
            },
        ];
        let out = entries(&groups, gw, gh, "test").expect("encode");
        assert_eq!(out.len(), 3, "two PNGs and one manifest");
        assert_eq!(out[2].name, "atlas/index.json");
        for e in &out[..2] {
            assert!(e.data.starts_with(&[0x89, b'P', b'N', b'G']), "{} is not a PNG", e.name);
            assert!(e.data.len() > 60, "{} is suspiciously small ({} bytes)", e.name, e.data.len());
        }
        let m = String::from_utf8(out[2].data.clone()).expect("utf-8");
        assert!(m.contains("\"cartalith-channel-atlas\""));
        assert!(m.contains("\"soil_fertility\"") && m.contains("\"biome\""));
        assert!(m.contains("\"kind\": \"index\"") && m.contains("\"kind\": \"unit\""));
        assert!(m.contains("\"biome_index.json\""));
        assert!(!m.contains("empty.png"), "a dropped group must not stay in the manifest");
        assert!(m.contains("\"width\": 8") && m.contains("\"height\": 5"));
    }

    #[test]
    fn an_atlas_of_nothing_is_no_files_rather_than_an_empty_manifest() {
        let groups = vec![ChannelGroup {
            file: "atlas/habitat.png".into(),
            channels: vec![Channel { ch: "r", key: "soil_fertility".into(), name: "Soil fertility".into(), src: None, manifest: None }],
        }];
        assert!(entries(&groups, 4, 4, "test").expect("encode").is_empty());
        // And a zero-size grid does not panic or divide by zero -- this runs
        // behind a `#[func]`.
        assert!(entries(&groups, 0, 0, "test").expect("encode").is_empty());
    }
}
